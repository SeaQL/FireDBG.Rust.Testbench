use core::fmt;
use core::str;

use memchr::memchr;

use crate::{QuoteStyle, Terminator};

/// A builder for configuring a CSV writer.
///
/// This builder permits specifying the CSV delimiter, terminator, quoting
/// style and more.
#[derive(Debug)]
pub struct WriterBuilder {
    wtr: Writer,
}

impl WriterBuilder {
    /// Create a new builder for configuring a CSV writer.
    pub fn new() -> WriterBuilder {
        let wtr = Writer {
            state: WriterState::default(),
            requires_quotes: [false; 256],
            delimiter: b',',
            term: Terminator::Any(b'\n'),
            style: QuoteStyle::default(),
            quote: b'"',
            escape: b'\\',
            double_quote: true,
            comment: None,
        };
        WriterBuilder { wtr: wtr }
    }

    /// Builder a CSV writer from this configuration.
    pub fn build(&self) -> Writer {
        use crate::Terminator::*;

        let mut wtr = self.wtr.clone();
        wtr.requires_quotes[self.wtr.delimiter as usize] = true;
        wtr.requires_quotes[self.wtr.quote as usize] = true;
        if !self.wtr.double_quote {
            // We only need to quote the escape character if the escape
            // character is used for escaping quotes.
            wtr.requires_quotes[self.wtr.escape as usize] = true;
        }
        match self.wtr.term {
            CRLF | Any(b'\n') | Any(b'\r') => {
                // This is a bit hokey. By default, the record terminator
                // is '\n', but we still need to quote '\r' (even if our
                // terminator is only `\n`) because the reader interprets '\r'
                // as a record terminator by default.
                wtr.requires_quotes[b'\r' as usize] = true;
                wtr.requires_quotes[b'\n' as usize] = true;
            }
            Any(b) => {
                wtr.requires_quotes[b as usize] = true;
            }
            _ => unreachable!(),
        }
        // If the first field of a row starts with a comment character,
        // it needs to be quoted, or the row will not be readable later.
        // As requires_quotes is calculated in advance, we force quotes
        // when a comment character is encountered anywhere in the field.
        if let Some(comment) = self.wtr.comment {
            wtr.requires_quotes[comment as usize] = true;
        }
        wtr
    }

    /// The field delimiter to use when writing CSV.
    ///
    /// The default is `b','`.
    pub fn delimiter(&mut self, delimiter: u8) -> &mut WriterBuilder {
        self.wtr.delimiter = delimiter;
        self
    }

    /// The record terminator to use when writing CSV.
    ///
    /// A record terminator can be any single byte. The default is `\n`.
    ///
    /// Note that RFC 4180 specifies that record terminators should be `\r\n`.
    /// To use `\r\n`, use the special `Terminator::CRLF` value.
    pub fn terminator(&mut self, term: Terminator) -> &mut WriterBuilder {
        self.wtr.term = term;
        self
    }

    /// The quoting style to use when writing CSV.
    ///
    /// By default, this is set to `QuoteStyle::Necessary`, which will only
    /// use quotes when they are necessary to preserve the integrity of data.
    ///
    /// Note that unless the quote style is set to `Never`, an empty field is
    /// quoted if it is the only field in a record.
    pub fn quote_style(&mut self, style: QuoteStyle) -> &mut WriterBuilder {
        self.wtr.style = style;
        self
    }

    /// The quote character to use when writing CSV.
    ///
    /// The default value is `b'"'`.
    pub fn quote(&mut self, quote: u8) -> &mut WriterBuilder {
        self.wtr.quote = quote;
        self
    }

    /// The escape character to use when writing CSV.
    ///
    /// This is only used when `double_quote` is set to `false`.
    ///
    /// The default value is `b'\\'`.
    pub fn escape(&mut self, escape: u8) -> &mut WriterBuilder {
        self.wtr.escape = escape;
        self
    }

    /// The quoting escape mechanism to use when writing CSV.
    ///
    /// When enabled (which is the default), quotes are escaped by doubling
    /// them. e.g., `"` escapes to `""`.
    ///
    /// When disabled, quotes are escaped with the escape character (which
    /// is `\\` by default).
    pub fn double_quote(&mut self, yes: bool) -> &mut WriterBuilder {
        self.wtr.double_quote = yes;
        self
    }

    /// The comment character that will be used when later reading the file.
    ///
    /// If `quote_style` is set to `QuoteStyle::Necessary`, a field will
    /// be quoted if the comment character is detected anywhere in the field.
    ///
    /// The default value is None.
    pub fn comment(&mut self, comment: Option<u8>) -> &mut WriterBuilder {
        self.wtr.comment = comment;
        self
    }
}

impl Default for WriterBuilder {
    fn default() -> WriterBuilder {
        WriterBuilder::new()
    }
}

/// The result of writing CSV data.
///
/// A value of this type is returned from every interaction with `Writer`. It
/// informs the caller how to proceed, namely, by indicating whether more
/// input should be given (`InputEmpty`) or if a bigger output buffer is needed
/// (`OutputFull`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WriteResult {
    /// This result occurs when all of the bytes from the given input have
    /// been processed.
    InputEmpty,
    /// This result occurs when the output buffer was too small to process
    /// all of the input bytes. Generally, this means the caller must call
    /// the corresponding method again with the rest of the input and more
    /// room in the output buffer.
    OutputFull,
}

/// A writer for CSV data.
///
/// # RFC 4180
///
/// This writer conforms to RFC 4180 with one exception: it doesn't guarantee
/// that all records written are of the same length. Instead, the onus is on
/// the caller to ensure that all records written are of the same length.
///
/// Note that the default configuration of a `Writer` uses `\n` for record
/// terminators instead of `\r\n` as specified by RFC 4180. Use the
/// `terminator` method on `WriterBuilder` to set the terminator to `\r\n` if
/// it's desired.
pub struct Writer {
    state: WriterState,
    requires_quotes: [bool; 256],
    delimiter: u8,
    term: Terminator,
    style: QuoteStyle,
    quote: u8,
    escape: u8,
    double_quote: bool,
    comment: Option<u8>,
}

impl Clone for Writer {
    fn clone(&self) -> Writer {
        let mut requires_quotes = [false; 256];
        for i in 0..256 {
            requires_quotes[i] = self.requires_quotes[i];
        }
        Writer {
            state: self.state.clone(),
            requires_quotes: requires_quotes,
            delimiter: self.delimiter,
            term: self.term,
            style: self.style,
            quote: self.quote,
            escape: self.escape,
            double_quote: self.double_quote,
            comment: self.comment,
        }
    }
}

impl fmt::Debug for Writer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Writer")
            .field("state", &self.state)
            .field("delimiter", &self.delimiter)
            .field("term", &self.term)
            .field("style", &self.style)
            .field("quote", &self.quote)
            .field("escape", &self.escape)
            .field("double_quote", &self.double_quote)
            .finish()
    }
}

#[derive(Clone, Debug)]
struct WriterState {
    /// This is set whenever we've begun writing the contents of a field, even
    /// if the contents are empty. We use it to avoid re-computing whether
    /// quotes are necessary.
    in_field: bool,
    /// This is set whenever we've started writing a field that is enclosed in
    /// quotes. When the writer is finished, or if a delimiter or terminator
    /// are written, then a closing quote is inserted when this is true.
    quoting: bool,
    /// The number of total bytes written for the current record.
    ///
    /// If the writer is finished or a terminator is written when this is `0`,
    /// then an empty field is added as a pair of adjacent quotes.
    record_bytes: u64,
}

impl Writer {
    /// Creates a new CSV writer with the default configuration.
    pub fn new() -> Writer {
        Writer::default()
    }

    /// Finish writing CSV data to `output`.
    ///
    /// This must be called when one is done writing CSV data to `output`.
    /// In particular, it will write closing quotes if necessary.
    pub fn finish(&mut self, mut output: &mut [u8]) -> (WriteResult, usize) {
        let mut nout = 0;
        if self.state.record_bytes == 0 && self.state.in_field {
            assert!(!self.state.quoting);
            let (res, o) = self.write(&[self.quote, self.quote], output);
            if o == 0 {
                return (res, 0);
            }
            output = &mut moving(output)[o..];
            nout += o;
            self.state.record_bytes += o as u64;
        }
        if !self.state.quoting {
            return (WriteResult::InputEmpty, nout);
        }
        let (res, o) = self.write(&[self.quote], output);
        if o == 0 {
            return (res, nout);
        }
        nout += o;
        self.state.record_bytes = 0;
        self.state.in_field = false;
        self.state.quoting = false;
        (res, nout)
    }

    /// Write a single CSV field from `input` to `output` while employing this
    /// writer's quoting style.
    ///
    /// This returns the result of writing field data, in addition to the
    /// number of bytes consumed from `input` and the number of bytes
    /// written to `output`.
    ///
    /// The result of writing field data is either `WriteResult::InputEmpty`
    /// or `WriteResult::OutputFull`. The former occurs when all bytes in
    /// `input` were copied to `output`, while the latter occurs when `output`
    /// is too small to fit everything from `input`. The maximum number of
    /// bytes that can be written to `output` is `2 + (2 * input.len())`
    /// because of quoting. (The worst case is a field consisting entirely
    /// of quotes.)
    ///
    /// Multiple successive calls to `field` will write more data to the same
    /// field. Subsequent fields can be written by calling either `delimiter`
    /// or `terminator` first.
    ///
    /// If this writer's quoting style is `QuoteStyle::Necessary`, then `input`
    /// should contain the *entire* field. Otherwise, whether the field needs
    /// to be quoted or not cannot be determined.
    pub fn field(
        &mut self,
        input: &[u8],
        mut output: &mut [u8],
    ) -> (WriteResult, usize, usize) {
        let (mut nin, mut nout) = (0, 0);

        if !self.state.in_field {
            self.state.quoting = self.should_quote(input);
            if self.state.quoting {
                let (res, o) = self.write(&[self.quote], output);
                if o == 0 {
                    return (res, 0, 0);
                }
                output = &mut moving(output)[o..];
                nout += o;
                self.state.record_bytes += o as u64;
            }
            self.state.in_field = true;
        }
        let (res, i, o) = if self.state.quoting {
            quote(input, output, self.quote, self.escape, self.double_quote)
        } else {
            write_optimistic(input, output)
        };
        nin += i;
        nout += o;
        self.state.record_bytes += o as u64;
        (res, nin, nout)
    }

    /// Write the configured field delimiter to `output`.
    ///
    /// If the output buffer does not have enough room to fit
    /// a field delimiter, then nothing is written to `output`
    /// and `WriteResult::OutputFull` is returned. Otherwise,
    /// `WriteResult::InputEmpty` is returned along with the number of bytes
    /// written to `output` (which is `1` in case of an unquoted
    /// field, or `2` in case of an end quote and a field separator).
    pub fn delimiter(
        &mut self,
        mut output: &mut [u8],
    ) -> (WriteResult, usize) {
        let mut nout = 0;
        if self.state.quoting {
            let (res, o) = self.write(&[self.quote], output);
            if o == 0 {
                return (res, o);
            }
            output = &mut moving(output)[o..];
            nout += o;
            self.state.record_bytes += o as u64;
            self.state.quoting = false;
        }
        let (res, o) = self.write(&[self.delimiter], output);
        if o == 0 {
            return (res, nout);
        }
        nout += o;
        self.state.record_bytes += o as u64;
        self.state.in_field = false;
        (res, nout)
    }

    /// Write the configured record terminator to `output`.
    ///
    /// If the output buffer does not have enough room to fit a record
    /// terminator, then no part of the terminator is written and
    /// `WriteResult::OutputFull` is returned. Otherwise,
    /// `WriteResult::InputEmpty` is returned along with the number of bytes
    /// written to `output` (which is always `1` or `2`).
    pub fn terminator(
        &mut self,
        mut output: &mut [u8],
    ) -> (WriteResult, usize) {
        let mut nout = 0;
        if self.state.record_bytes == 0 {
            assert!(!self.state.quoting);
            let (res, o) = self.write(&[self.quote, self.quote], output);
            if o == 0 {
                return (res, 0);
            }
            output = &mut moving(output)[o..];
            nout += o;
            self.state.record_bytes += o as u64;
        }
        if self.state.quoting {
            let (res, o) = self.write(&[self.quote], output);
            if o == 0 {
                return (res, o);
            }
            output = &mut moving(output)[o..];
            nout += o;
            self.state.record_bytes += o as u64;
            self.state.quoting = false;
        }
        let (res, o) = match self.term {
            Terminator::CRLF => write_pessimistic(&[b'\r', b'\n'], output),
            Terminator::Any(b) => write_pessimistic(&[b], output),
            _ => unreachable!(),
        };
        if o == 0 {
            return (res, nout);
        }
        nout += o;
        self.state.record_bytes = 0;
        self.state.in_field = false;
        (res, nout)
    }

    /// Returns true if and only if the given input field *requires* quotes to
    /// preserve the integrity of `input` while taking into account the current
    /// configuration of this writer (except for the configured quoting style).
    #[inline]
    fn needs_quotes(&self, mut input: &[u8]) -> bool {
        let mut needs = false;
        while !needs && input.len() >= 8 {
            needs = self.requires_quotes[input[0] as usize]
                || self.requires_quotes[input[1] as usize]
                || self.requires_quotes[input[2] as usize]
                || self.requires_quotes[input[3] as usize]
                || self.requires_quotes[input[4] as usize]
                || self.requires_quotes[input[5] as usize]
                || self.requires_quotes[input[6] as usize]
                || self.requires_quotes[input[7] as usize];
            input = &input[8..];
        }
        needs || input.iter().any(|&b| self.is_special_byte(b))
    }

    /// Returns true if and only if the given byte corresponds to a special
    /// byte in this CSV writer's configuration.
    ///
    /// Note that this does **not** take into account this writer's quoting
    /// style.
    #[inline]
    pub fn is_special_byte(&self, b: u8) -> bool {
        self.requires_quotes[b as usize]
    }

    /// Returns true if and only if we should put the given field data
    /// in quotes. This takes the quoting style into account.
    #[inline]
    pub fn should_quote(&self, input: &[u8]) -> bool {
        match self.style {
            QuoteStyle::Always => true,
            QuoteStyle::Never => false,
            QuoteStyle::NonNumeric => is_non_numeric(input),
            QuoteStyle::Necessary => self.needs_quotes(input),
            _ => unreachable!(),
        }
    }

    /// Return the delimiter used for this writer.
    #[inline]
    pub fn get_delimiter(&self) -> u8 {
        self.delimiter
    }

    /// Return the terminator used for this writer.
    #[inline]
    pub fn get_terminator(&self) -> Terminator {
        self.term
    }

    /// Return the quoting style used for this writer.
    #[inline]
    pub fn get_quote_style(&self) -> QuoteStyle {
        self.style
    }

    /// Return the quote character used for this writer.
    #[inline]
    pub fn get_quote(&self) -> u8 {
        self.quote
    }

    /// Return the escape character used for this writer.
    #[inline]
    pub fn get_escape(&self) -> u8 {
        self.escape
    }

    /// Return whether this writer doubles quotes or not. When the writer
    /// does not double quotes, it will escape them using the escape character.
    #[inline]
    pub fn get_double_quote(&self) -> bool {
        self.double_quote
    }

    fn write(&self, data: &[u8], output: &mut [u8]) -> (WriteResult, usize) {
        if data.len() > output.len() {
            (WriteResult::OutputFull, 0)
        } else {
            output[..data.len()].copy_from_slice(data);
            (WriteResult::InputEmpty, data.len())
        }
    }
}

impl Default for Writer {
    fn default() -> Writer {
        WriterBuilder::new().build()
    }
}

impl Default for WriterState {
    fn default() -> WriterState {
        WriterState { in_field: false, quoting: false, record_bytes: 0 }
    }
}

/// Returns true if and only if the given input is non-numeric.
pub fn is_non_numeric(input: &[u8]) -> bool {
    let s = match str::from_utf8(input) {
        Err(_) => return true,
        Ok(s) => s,
    };
    // I suppose this could be faster if we wrote validators of numbers instead
    // of using the actual parser, but that's probably a lot of work for a bit
    // of a niche feature.
    !s.parse::<f64>().is_ok() && !s.parse::<i128>().is_ok()
}

/// Escape quotes `input` and writes the result to `output`.
///
/// If `input` does not have a `quote`, then the contents of `input` are
/// copied verbatim to `output`.
///
/// If `output` is not big enough to store the fully quoted contents of
/// `input`, then `WriteResult::OutputFull` is returned. The `output` buffer
/// will require a maximum of storage of `2 * input.len()` in the worst case
/// (where every byte is a quote).
///
/// In streaming contexts, `quote` should be called in a loop until
/// `WriteResult::InputEmpty` is returned. It is possible to write an infinite
/// loop if your output buffer is less than 2 bytes in length (the minimum
/// storage space required to store an escaped quote).
///
/// In addition to the `WriteResult`, the number of consumed bytes from `input`
/// and the number of bytes written to `output` are also returned.
///
/// `quote` is the quote byte and `escape` is the escape byte. If
/// `double_quote` is true, then quotes are escaped by doubling them,
/// otherwise, quotes are escaped with the `escape` byte.
///
/// N.B. This function is provided for low level usage. It is called
/// automatically if you're using a `Writer`.
pub fn quote(
    mut input: &[u8],
    mut output: &mut [u8],
    quote: u8,
    escape: u8,
    double_quote: bool,
) -> (WriteResult, usize, usize) {
    let (mut nin, mut nout) = (0, 0);
    loop {
        match memchr(quote, input) {
            None => {
                let (res, i, o) = write_optimistic(input, output);
                nin += i;
                nout += o;
                return (res, nin, nout);
            }
            Some(next_quote) => {
                let (res, i, o) =
                    write_optimistic(&input[..next_quote], output);
                input = &input[i..];
                output = &mut moving(output)[o..];
                nin += i;
                nout += o;
                if let WriteResult::OutputFull = res {
                    return (res, nin, nout);
                }
                if double_quote {
                    let (res, o) = write_pessimistic(&[quote, quote], output);
                    if let WriteResult::OutputFull = res {
                        return (res, nin, nout);
                    }
                    nout += o;
                    output = &mut moving(output)[o..];
                } else {
                    let (res, o) = write_pessimistic(&[escape, quote], output);
                    if let WriteResult::OutputFull = res {
                        return (res, nin, nout);
                    }
                    nout += o;
                    output = &mut moving(output)[o..];
                }
                nin += 1;
                input = &input[1..];
            }
        }
    }
}

/// Copy the bytes from `input` to `output`. If `output` is too small to fit
/// everything from `input`, then copy `output.len()` bytes from `input`.
/// Otherwise, copy everything from `input` into `output`.
///
/// In the first case (`output` is too small), `WriteResult::OutputFull` is
/// returned, in addition to the number of bytes consumed from `input` and
/// the number of bytes written to `output`.
///
/// In the second case (`input` is no bigger than `output`),
/// `WriteResult::InputEmpty` is returned, in addition to the number of bytes
/// consumed from `input` and the number of bytes written to `output`.
fn write_optimistic(
    input: &[u8],
    output: &mut [u8],
) -> (WriteResult, usize, usize) {
    if input.len() > output.len() {
        let input = &input[..output.len()];
        output.copy_from_slice(input);
        (WriteResult::OutputFull, output.len(), output.len())
    } else {
        output[..input.len()].copy_from_slice(input);
        (WriteResult::InputEmpty, input.len(), input.len())
    }
}

/// Copy the bytes from `input` to `output` only if `input` is no bigger than
/// `output`. If `input` is bigger than `output`, then return
/// `WriteResult::OutputFull` and copy nothing into `output`. Otherwise,
/// return `WriteResult::InputEmpty` and the number of bytes copied into
/// `output`.
fn write_pessimistic(input: &[u8], output: &mut [u8]) -> (WriteResult, usize) {
    if input.len() > output.len() {
        (WriteResult::OutputFull, 0)
    } else {
        output[..input.len()].copy_from_slice(input);
        (WriteResult::InputEmpty, input.len())
    }
}

/// This avoids reborrowing.
/// See: https://bluss.github.io/rust/fun/2015/10/11/stuff-the-identity-function-does/
fn moving<T>(x: T) -> T {
    x
}
