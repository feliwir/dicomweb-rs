//! MultipartRelated response payload support.

use std::{
    cell::{Cell, RefCell, RefMut},
    cmp, fmt,
    marker::PhantomData,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use actix_multipart::MultipartError;
use actix_web::{
    error::{ParseError, PayloadError},
    http::header::{self, ContentDisposition, HeaderMap, HeaderName, HeaderValue},
    web::{Bytes, BytesMut},
};
use futures_util::stream::{LocalBoxStream, Stream};
use local_waker::LocalWaker;

const MAX_HEADERS: usize = 32;

/// The server-side implementation of `multipart/related` requests.
///
/// This will parse the incoming stream into `MultipartItem` instances via its
/// Stream implementation.
/// `MultipartItem::Object` contains multipart field. `MultipartItem::MultipartRelated`
/// is used for nested multipart streams.
pub struct MultipartRelated {
    safety: Safety,
    error: Option<MultipartError>,
    inner: Option<InnerMultipart>,
}

enum InnerMultipartItem {
    None,
    Object(Rc<RefCell<InnerField>>),
}

#[derive(PartialEq, Debug)]
enum InnerState {
    /// Stream eof
    Eof,

    /// Skip data until first boundary
    FirstBoundary,

    /// Reading boundary
    Boundary,

    /// Reading Headers,
    Headers,
}

struct InnerMultipart {
    payload: PayloadRef,
    boundary: String,
    state: InnerState,
    item: InnerMultipartItem,
}

impl MultipartRelated {
    /// Create multipart instance for boundary.
    pub fn new<S>(headers: &HeaderMap, stream: S) -> MultipartRelated
    where
        S: Stream<Item = Result<Bytes, PayloadError>> + 'static,
    {
        match Self::boundary(headers) {
            Ok(boundary) => MultipartRelated::from_boundary(boundary, stream),
            Err(err) => MultipartRelated::from_error(err),
        }
    }

    /// Extract boundary info from headers.
    pub(crate) fn boundary(headers: &HeaderMap) -> Result<String, MultipartError> {
        headers
            .get(&header::CONTENT_TYPE)
            .ok_or(MultipartError::NoContentType)?
            .to_str()
            .ok()
            .and_then(|content_type| content_type.parse::<mime::Mime>().ok())
            .ok_or(MultipartError::ParseContentType)?
            .get_param(mime::BOUNDARY)
            .map(|boundary| boundary.as_str().to_owned())
            .ok_or(MultipartError::Boundary)
    }

    /// Create multipart instance for given boundary and stream
    pub(crate) fn from_boundary<S>(boundary: String, stream: S) -> MultipartRelated
    where
        S: Stream<Item = Result<Bytes, PayloadError>> + 'static,
    {
        MultipartRelated {
            error: None,
            safety: Safety::new(),
            inner: Some(InnerMultipart {
                boundary,
                payload: PayloadRef::new(PayloadBuffer::new(stream)),
                state: InnerState::FirstBoundary,
                item: InnerMultipartItem::None,
            }),
        }
    }

    /// Create MultipartRelated instance from MultipartError
    pub(crate) fn from_error(err: MultipartError) -> MultipartRelated {
        MultipartRelated {
            error: Some(err),
            safety: Safety::new(),
            inner: None,
        }
    }
}

impl Stream for MultipartRelated {
    type Item = Result<Object, MultipartError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        match this.inner.as_mut() {
            Some(inner) => {
                if let Some(mut buffer) = inner.payload.get_mut(&this.safety) {
                    // check safety and poll read payload to buffer.
                    buffer.poll_stream(cx)?;
                } else if !this.safety.is_clean() {
                    // safety violation
                    return Poll::Ready(Some(Err(MultipartError::NotConsumed)));
                } else {
                    return Poll::Pending;
                }

                inner.poll(&this.safety, cx)
            }
            None => Poll::Ready(Some(Err(this
                .error
                .take()
                .expect("MultipartRelated polled after finish")))),
        }
    }
}

impl InnerMultipart {
    fn read_headers(payload: &mut PayloadBuffer) -> Result<Option<HeaderMap>, MultipartError> {
        match payload.read_until(b"\r\n\r\n")? {
            None => {
                if payload.eof {
                    Err(MultipartError::Incomplete)
                } else {
                    Ok(None)
                }
            }
            Some(bytes) => {
                let mut hdrs = [httparse::EMPTY_HEADER; MAX_HEADERS];
                match httparse::parse_headers(&bytes, &mut hdrs) {
                    Ok(httparse::Status::Complete((_, hdrs))) => {
                        // convert headers
                        let mut headers = HeaderMap::with_capacity(hdrs.len());

                        for h in hdrs {
                            let name =
                                HeaderName::try_from(h.name).map_err(|_| ParseError::Header)?;
                            let value =
                                HeaderValue::try_from(h.value).map_err(|_| ParseError::Header)?;
                            headers.append(name, value);
                        }

                        Ok(Some(headers))
                    }
                    Ok(httparse::Status::Partial) => Err(ParseError::Header.into()),
                    Err(err) => Err(ParseError::from(err).into()),
                }
            }
        }
    }

    fn read_boundary(
        payload: &mut PayloadBuffer,
        boundary: &str,
    ) -> Result<Option<bool>, MultipartError> {
        // TODO: need to read epilogue
        match payload.readline_or_eof()? {
            None => {
                if payload.eof {
                    Ok(Some(true))
                } else {
                    Ok(None)
                }
            }
            Some(chunk) => {
                if chunk.len() < boundary.len() + 4
                    || &chunk[..2] != b"--"
                    || &chunk[2..boundary.len() + 2] != boundary.as_bytes()
                {
                    Err(MultipartError::Boundary)
                } else if &chunk[boundary.len() + 2..] == b"\r\n" {
                    Ok(Some(false))
                } else if &chunk[boundary.len() + 2..boundary.len() + 4] == b"--"
                    && (chunk.len() == boundary.len() + 4
                        || &chunk[boundary.len() + 4..] == b"\r\n")
                {
                    Ok(Some(true))
                } else {
                    Err(MultipartError::Boundary)
                }
            }
        }
    }

    fn skip_until_boundary(
        payload: &mut PayloadBuffer,
        boundary: &str,
    ) -> Result<Option<bool>, MultipartError> {
        let mut eof = false;
        loop {
            match payload.readline()? {
                Some(chunk) => {
                    if chunk.is_empty() {
                        return Err(MultipartError::Boundary);
                    }
                    if chunk.len() < boundary.len() {
                        continue;
                    }
                    if &chunk[..2] == b"--" && &chunk[2..chunk.len() - 2] == boundary.as_bytes() {
                        break;
                    } else {
                        if chunk.len() < boundary.len() + 2 {
                            continue;
                        }
                        let b: &[u8] = boundary.as_ref();
                        if &chunk[..boundary.len()] == b
                            && &chunk[boundary.len()..boundary.len() + 2] == b"--"
                        {
                            eof = true;
                            break;
                        }
                    }
                }
                None => {
                    return if payload.eof {
                        Err(MultipartError::Incomplete)
                    } else {
                        Ok(None)
                    };
                }
            }
        }
        Ok(Some(eof))
    }

    fn poll(
        &mut self,
        safety: &Safety,
        cx: &Context<'_>,
    ) -> Poll<Option<Result<Object, MultipartError>>> {
        if self.state == InnerState::Eof {
            Poll::Ready(None)
        } else {
            // release field
            loop {
                // Nested multipart streams of fields has to be consumed
                // before switching to next
                if safety.current() {
                    let stop = match self.item {
                        InnerMultipartItem::Object(ref mut field) => {
                            match field.borrow_mut().poll(safety) {
                                Poll::Pending => return Poll::Pending,
                                Poll::Ready(Some(Ok(_))) => continue,
                                Poll::Ready(Some(Err(err))) => return Poll::Ready(Some(Err(err))),
                                Poll::Ready(None) => true,
                            }
                        }
                        InnerMultipartItem::None => false,
                    };
                    if stop {
                        self.item = InnerMultipartItem::None;
                    }
                    if let InnerMultipartItem::None = self.item {
                        break;
                    }
                }
            }

            let headers = if let Some(mut payload) = self.payload.get_mut(safety) {
                match self.state {
                    // read until first boundary
                    InnerState::FirstBoundary => {
                        match InnerMultipart::skip_until_boundary(&mut payload, &self.boundary)? {
                            Some(eof) => {
                                if eof {
                                    self.state = InnerState::Eof;
                                    return Poll::Ready(None);
                                } else {
                                    self.state = InnerState::Headers;
                                }
                            }
                            None => return Poll::Pending,
                        }
                    }
                    // read boundary
                    InnerState::Boundary => {
                        match InnerMultipart::read_boundary(&mut payload, &self.boundary)? {
                            None => return Poll::Pending,
                            Some(eof) => {
                                if eof {
                                    self.state = InnerState::Eof;
                                    return Poll::Ready(None);
                                } else {
                                    self.state = InnerState::Headers;
                                }
                            }
                        }
                    }
                    _ => {}
                }

                // read field headers for next field
                if self.state == InnerState::Headers {
                    if let Some(headers) = InnerMultipart::read_headers(&mut payload)? {
                        self.state = InnerState::Boundary;
                        headers
                    } else {
                        return Poll::Pending;
                    }
                } else {
                    unreachable!()
                }
            } else {
                log::debug!("NotReady: field is in flight");
                return Poll::Pending;
            };

            let cd = headers
                .get(&header::CONTENT_DISPOSITION)
                .and_then(|cd| ContentDisposition::from_raw(cd).ok());

            let ct: Option<mime::Mime> = headers
                .get(&header::CONTENT_TYPE)
                .and_then(|ct| ct.to_str().ok())
                .and_then(|ct| ct.parse().ok());

            self.state = InnerState::Boundary;

            // nested multipart stream is not supported
            if let Some(mime) = &ct {
                if mime.type_() == mime::MULTIPART {
                    return Poll::Ready(Some(Err(MultipartError::Nested)));
                }
            }

            let field =
                InnerField::new_in_rc(self.payload.clone(), self.boundary.clone(), &headers)?;

            self.item = InnerMultipartItem::Object(Rc::clone(&field));

            Poll::Ready(Some(Ok(Object::new(
                safety.clone(cx),
                headers,
                ct,
                cd,
                field,
            ))))
        }
    }
}

impl Drop for InnerMultipart {
    fn drop(&mut self) {
        // InnerMultipartItem::Object has to be dropped first because of Safety.
        self.item = InnerMultipartItem::None;
    }
}

/// A single field in a multipart stream
pub struct Object {
    ct: Option<mime::Mime>,
    cd: Option<ContentDisposition>,
    headers: HeaderMap,
    inner: Rc<RefCell<InnerField>>,
    safety: Safety,
}

impl Object {
    fn new(
        safety: Safety,
        headers: HeaderMap,
        ct: Option<mime::Mime>,
        cd: Option<ContentDisposition>,
        inner: Rc<RefCell<InnerField>>,
    ) -> Self {
        Object {
            ct,
            cd,
            headers,
            inner,
            safety,
        }
    }

    /// Returns a reference to the field's header map.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Returns a reference to the field's content (mime) type, if it is supplied by the client.
    ///
    /// According to [RFC 7578](https://www.rfc-editor.org/rfc/rfc7578#section-4.4), if it is not
    /// present, it should default to "text/plain". Note it is the responsibility of the client to
    /// provide the appropriate content type, there is no attempt to validate this by the server.
    pub fn content_type(&self) -> Option<&mime::Mime> {
        self.ct.as_ref()
    }

    /// Returns the field's Content-Disposition.
    ///
    /// Per [RFC 7578 §4.2]: "Each part MUST contain a Content-Disposition header field where the
    /// disposition type is `form-data`. The Content-Disposition header field MUST also contain an
    /// additional parameter of `name`; the value of the `name` parameter is the original field name
    /// from the form."
    ///
    /// This crate validates that it exists before returning a `Object`. As such, it is safe to
    /// unwrap `.content_disposition().get_name()`. The [name](Self::name) method is provided as
    /// a convenience.
    ///
    /// [RFC 7578 §4.2]: https://datatracker.ietf.org/doc/html/rfc7578#section-4.2
    pub fn content_disposition(&self) -> Option<&ContentDisposition> {
        self.cd.as_ref()
    }
}

impl Stream for Object {
    type Item = Result<Bytes, MultipartError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let mut inner = this.inner.borrow_mut();
        if let Some(mut buffer) = inner.payload.as_ref().unwrap().get_mut(&this.safety) {
            // check safety and poll read payload to buffer.
            buffer.poll_stream(cx)?;
        } else if !this.safety.is_clean() {
            // safety violation
            return Poll::Ready(Some(Err(MultipartError::NotConsumed)));
        } else {
            return Poll::Pending;
        }

        inner.poll(&this.safety)
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ct) = &self.ct {
            writeln!(f, "\nField: {}", ct)?;
        } else {
            writeln!(f, "\nField:")?;
        }
        writeln!(f, "  boundary: {}", self.inner.borrow().boundary)?;
        writeln!(f, "  headers:")?;
        for (key, val) in self.headers.iter() {
            writeln!(f, "    {:?}: {:?}", key, val)?;
        }
        Ok(())
    }
}

struct InnerField {
    payload: Option<PayloadRef>,
    boundary: String,
    eof: bool,
    length: Option<u64>,
}

impl InnerField {
    fn new_in_rc(
        payload: PayloadRef,
        boundary: String,
        headers: &HeaderMap,
    ) -> Result<Rc<RefCell<InnerField>>, PayloadError> {
        Self::new(payload, boundary, headers).map(|this| Rc::new(RefCell::new(this)))
    }

    fn new(
        payload: PayloadRef,
        boundary: String,
        headers: &HeaderMap,
    ) -> Result<InnerField, PayloadError> {
        let len = if let Some(len) = headers.get(&header::CONTENT_LENGTH) {
            match len.to_str().ok().and_then(|len| len.parse::<u64>().ok()) {
                Some(len) => Some(len),
                None => return Err(PayloadError::Incomplete(None)),
            }
        } else {
            None
        };

        Ok(InnerField {
            boundary,
            payload: Some(payload),
            eof: false,
            length: len,
        })
    }

    /// Reads body part content chunk of the specified size.
    /// The body part must has `Content-Length` header with proper value.
    fn read_len(
        payload: &mut PayloadBuffer,
        size: &mut u64,
    ) -> Poll<Option<Result<Bytes, MultipartError>>> {
        if *size == 0 {
            Poll::Ready(None)
        } else {
            match payload.read_max(*size)? {
                Some(mut chunk) => {
                    let len = cmp::min(chunk.len() as u64, *size);
                    *size -= len;
                    let ch = chunk.split_to(len as usize);
                    if !chunk.is_empty() {
                        payload.unprocessed(chunk);
                    }
                    Poll::Ready(Some(Ok(ch)))
                }
                None => {
                    if payload.eof && (*size != 0) {
                        Poll::Ready(Some(Err(MultipartError::Incomplete)))
                    } else {
                        Poll::Pending
                    }
                }
            }
        }
    }

    /// Reads content chunk of body part with unknown length.
    /// The `Content-Length` header for body part is not necessary.
    fn read_stream(
        payload: &mut PayloadBuffer,
        boundary: &str,
    ) -> Poll<Option<Result<Bytes, MultipartError>>> {
        let mut pos = 0;

        let len = payload.buf.len();
        if len == 0 {
            return if payload.eof {
                Poll::Ready(Some(Err(MultipartError::Incomplete)))
            } else {
                Poll::Pending
            };
        }

        // check boundary
        if len > 4 && payload.buf[0] == b'\r' {
            let b_len = if &payload.buf[..2] == b"\r\n" && &payload.buf[2..4] == b"--" {
                Some(4)
            } else if &payload.buf[1..3] == b"--" {
                Some(3)
            } else {
                None
            };

            if let Some(b_len) = b_len {
                let b_size = boundary.len() + b_len;
                if len < b_size {
                    return Poll::Pending;
                } else if &payload.buf[b_len..b_size] == boundary.as_bytes() {
                    // found boundary
                    return Poll::Ready(None);
                }
            }
        }

        loop {
            return if let Some(idx) = memchr::memmem::find(&payload.buf[pos..], b"\r") {
                let cur = pos + idx;

                // check if we have enough data for boundary detection
                if cur + 4 > len {
                    if cur > 0 {
                        Poll::Ready(Some(Ok(payload.buf.split_to(cur).freeze())))
                    } else {
                        Poll::Pending
                    }
                } else {
                    // check boundary
                    if (&payload.buf[cur..cur + 2] == b"\r\n"
                        && &payload.buf[cur + 2..cur + 4] == b"--")
                        || (&payload.buf[cur..=cur] == b"\r"
                            && &payload.buf[cur + 1..cur + 3] == b"--")
                    {
                        if cur != 0 {
                            // return buffer
                            Poll::Ready(Some(Ok(payload.buf.split_to(cur).freeze())))
                        } else {
                            pos = cur + 1;
                            continue;
                        }
                    } else {
                        // not boundary
                        pos = cur + 1;
                        continue;
                    }
                }
            } else {
                Poll::Ready(Some(Ok(payload.buf.split().freeze())))
            };
        }
    }

    fn poll(&mut self, s: &Safety) -> Poll<Option<Result<Bytes, MultipartError>>> {
        if self.payload.is_none() {
            return Poll::Ready(None);
        }

        let result = if let Some(mut payload) = self.payload.as_ref().unwrap().get_mut(s) {
            if !self.eof {
                let res = if let Some(ref mut len) = self.length {
                    InnerField::read_len(&mut payload, len)
                } else {
                    InnerField::read_stream(&mut payload, &self.boundary)
                };

                match res {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(Some(Ok(bytes))) => return Poll::Ready(Some(Ok(bytes))),
                    Poll::Ready(Some(Err(err))) => return Poll::Ready(Some(Err(err))),
                    Poll::Ready(None) => self.eof = true,
                }
            }

            match payload.readline() {
                Ok(None) => Poll::Pending,
                Ok(Some(line)) => {
                    if line.as_ref() != b"\r\n" {
                        log::warn!("multipart field did not read all the data or it is malformed");
                    }
                    Poll::Ready(None)
                }
                Err(err) => Poll::Ready(Some(Err(err))),
            }
        } else {
            Poll::Pending
        };

        if let Poll::Ready(None) = result {
            self.payload.take();
        }
        result
    }
}

struct PayloadRef {
    payload: Rc<RefCell<PayloadBuffer>>,
}

impl PayloadRef {
    fn new(payload: PayloadBuffer) -> PayloadRef {
        PayloadRef {
            payload: Rc::new(payload.into()),
        }
    }

    fn get_mut(&self, s: &Safety) -> Option<RefMut<'_, PayloadBuffer>> {
        if s.current() {
            Some(self.payload.borrow_mut())
        } else {
            None
        }
    }
}

impl Clone for PayloadRef {
    fn clone(&self) -> PayloadRef {
        PayloadRef {
            payload: Rc::clone(&self.payload),
        }
    }
}

/// Counter. It tracks of number of clones of payloads and give access to payload only to top most.
/// * When dropped, parent task is awakened. This is to support the case where Object is
/// dropped in a separate task than MultipartRelated.
/// * Assumes that parent owners don't move to different tasks; only the top-most is allowed to.
/// * If dropped and is not top most owner, is_clean flag is set to false.
#[derive(Debug)]
struct Safety {
    task: LocalWaker,
    level: usize,
    payload: Rc<PhantomData<bool>>,
    clean: Rc<Cell<bool>>,
}

impl Safety {
    fn new() -> Safety {
        let payload = Rc::new(PhantomData);
        Safety {
            task: LocalWaker::new(),
            level: Rc::strong_count(&payload),
            clean: Rc::new(Cell::new(true)),
            payload,
        }
    }

    fn current(&self) -> bool {
        Rc::strong_count(&self.payload) == self.level && self.clean.get()
    }

    fn is_clean(&self) -> bool {
        self.clean.get()
    }

    fn clone(&self, cx: &Context<'_>) -> Safety {
        let payload = Rc::clone(&self.payload);
        let s = Safety {
            task: LocalWaker::new(),
            level: Rc::strong_count(&payload),
            clean: self.clean.clone(),
            payload,
        };
        s.task.register(cx.waker());
        s
    }
}

impl Drop for Safety {
    fn drop(&mut self) {
        if Rc::strong_count(&self.payload) != self.level {
            // MultipartRelated dropped leaving a Object
            self.clean.set(false);
        }

        self.task.wake();
    }
}

/// Payload buffer.
struct PayloadBuffer {
    eof: bool,
    buf: BytesMut,
    stream: LocalBoxStream<'static, Result<Bytes, PayloadError>>,
}

impl PayloadBuffer {
    /// Constructs new `PayloadBuffer` instance.
    fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<Bytes, PayloadError>> + 'static,
    {
        PayloadBuffer {
            eof: false,
            buf: BytesMut::new(),
            stream: Box::pin(stream),
        }
    }

    fn poll_stream(&mut self, cx: &mut Context<'_>) -> Result<(), PayloadError> {
        loop {
            match Pin::new(&mut self.stream).poll_next(cx) {
                Poll::Ready(Some(Ok(data))) => self.buf.extend_from_slice(&data),
                Poll::Ready(Some(Err(err))) => return Err(err),
                Poll::Ready(None) => {
                    self.eof = true;
                    return Ok(());
                }
                Poll::Pending => return Ok(()),
            }
        }
    }

    fn read_max(&mut self, size: u64) -> Result<Option<Bytes>, MultipartError> {
        if !self.buf.is_empty() {
            let size = std::cmp::min(self.buf.len() as u64, size) as usize;
            Ok(Some(self.buf.split_to(size).freeze()))
        } else if self.eof {
            Err(MultipartError::Incomplete)
        } else {
            Ok(None)
        }
    }

    /// Read until specified ending
    fn read_until(&mut self, line: &[u8]) -> Result<Option<Bytes>, MultipartError> {
        let res = memchr::memmem::find(&self.buf, line)
            .map(|idx| self.buf.split_to(idx + line.len()).freeze());

        if res.is_none() && self.eof {
            Err(MultipartError::Incomplete)
        } else {
            Ok(res)
        }
    }

    /// Read bytes until new line delimiter
    fn readline(&mut self) -> Result<Option<Bytes>, MultipartError> {
        self.read_until(b"\n")
    }

    /// Read bytes until new line delimiter or eof
    fn readline_or_eof(&mut self) -> Result<Option<Bytes>, MultipartError> {
        match self.readline() {
            Err(MultipartError::Incomplete) if self.eof => Ok(Some(self.buf.split().freeze())),
            line => line,
        }
    }

    /// Put unprocessed data back to the buffer
    fn unprocessed(&mut self, data: Bytes) {
        let buf = BytesMut::from(data.as_ref());
        let buf = std::mem::replace(&mut self.buf, buf);
        self.buf.extend_from_slice(&buf);
    }
}
