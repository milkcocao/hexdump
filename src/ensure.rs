
use crate::Error;
use alloc::string::String;
use core::fmt::{self, Debug, Write};
use core::mem::MaybeUninit;
use core::ptr;
use core::slice;
use core::str;

#[doc(hidden)]
pub trait BothDebug {
    fn __dispatch_ensure(self, msg: &'static str) -> Error;
}

impl<A, B> BothDebug for (A, B)
where
    A: Debug,
    B: Debug,
{
    fn __dispatch_ensure(self, msg: &'static str) -> Error {
        render(msg, &self.0, &self.1)
    }
}

#[doc(hidden)]
pub trait NotBothDebug {
    fn __dispatch_ensure(self, msg: &'static str) -> Error;
}

impl<A, B> NotBothDebug for &(A, B) {
    fn __dispatch_ensure(self, msg: &'static str) -> Error {
        Error::msg(msg)
    }
}

struct Buf {
    bytes: [MaybeUninit<u8>; 40],
    written: usize,
}

impl Buf {
    fn new() -> Self {
        Buf {
            bytes: [MaybeUninit::uninit(); 40],
            written: 0,
        }
    }

    fn as_str(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(slice::from_raw_parts(
                self.bytes.as_ptr().cast::<u8>(),
                self.written,
            ))
        }
    }
}

impl Write for Buf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if s.bytes().any(|b| b == b' ' || b == b'\n') {
            return Err(fmt::Error);
        }

        let remaining = self.bytes.len() - self.written;
        if s.len() > remaining {
            return Err(fmt::Error);
        }

        unsafe {
            ptr::copy_nonoverlapping(
                s.as_ptr(),
                self.bytes.as_mut_ptr().add(self.written).cast::<u8>(),
                s.len(),
            );
        }
        self.written += s.len();
        Ok(())
    }
}

fn render(msg: &'static str, lhs: &dyn Debug, rhs: &dyn Debug) -> Error {
    let mut lhs_buf = Buf::new();
    if fmt::write(&mut lhs_buf, format_args!("{:?}", lhs)).is_ok() {
        let mut rhs_buf = Buf::new();
        if fmt::write(&mut rhs_buf, format_args!("{:?}", rhs)).is_ok() {
            let lhs_str = lhs_buf.as_str();
            let rhs_str = rhs_buf.as_str();
            // "{msg} ({lhs} vs {rhs})"
            let len = msg.len() + 2 + lhs_str.len() + 4 + rhs_str.len() + 1;
            let mut string = String::with_capacity(len);
            string.push_str(msg);
            string.push_str(" (");
            string.push_str(lhs_str);
            string.push_str(" vs ");
            string.push_str(rhs_str);
            string.push(')');
            return Error::msg(string);
        }
    }
    Error::msg(msg)
}

#[doc(hidden)]
#[macro_export]
macro_rules! __parse_ensure {
    (atom () $bail:tt $fuel:tt {($($rhs:tt)+) ($($lhs:tt)+) $op:tt} $dup:tt $(,)?) => {
        $crate::__fancy_ensure!($($lhs)+, $op, $($rhs)+)
    };

    // low precedence control flow constructs

    (0 $stack:tt ($($bail:tt)*) $fuel:tt $parse:tt $dup:tt return $($rest:tt)*) => {
        $crate::__fallback_ensure!($($bail)*)
    };

    (0 $stack:tt ($($bail:tt)*) $fuel:tt $parse:tt $dup:tt break $($rest:tt)*) => {
        $crate::__fallback_ensure!($($bail)*)
    };

    (0 $stack:tt ($($bail:tt)*) $fuel:tt $parse:tt $dup:tt continue $($rest:tt)*) => {
        $crate::__fallback_ensure!($($bail)*)
    };

    (0 $stack:tt ($($bail:tt)*) $fuel:tt $parse:tt $dup:tt yield $($rest:tt)*) => {
        $crate::__fallback_ensure!($($bail)*)
    };

    (0 $stack:tt ($($bail:tt)*) $fuel:tt $parse:tt $dup:tt move $($rest:tt)*) => {
        $crate::__fallback_ensure!($($bail)*)
    };

    // unary operators

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($deref:tt $($dup:tt)*) * $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $deref) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($not:tt $($dup:tt)*) ! $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $not) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($neg:tt $($dup:tt)*) - $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $neg) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($let:tt $($dup:tt)*) let $($rest:tt)*) => {
        $crate::__parse_ensure!(pat $stack $bail ($($fuel)*) {($($buf)* $let) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($life:tt $colon:tt $($dup:tt)*) $label:lifetime : $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $life $colon) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($and:tt $mut:tt $($dup:tt)*) &mut $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $and $mut) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($and:tt $($dup:tt)*) & $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $and) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($andand:tt $mut:tt $($dup:tt)*) &&mut $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $andand $mut) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($andand:tt $($dup:tt)*) && $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $andand) $($parse)*} ($($rest)*) $($rest)*)
    };

    // control flow constructs

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($if:tt $($dup:tt)*) if $($rest:tt)*) => {
        $crate::__parse_ensure!(0 (cond $stack) $bail ($($fuel)*) {($($buf)* $if) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($match:tt $($dup:tt)*) match $($rest:tt)*) => {
        $crate::__parse_ensure!(0 (cond $stack) $bail ($($fuel)*) {($($buf)* $match) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($while:tt $($dup:tt)*) while $($rest:tt)*) => {
        $crate::__parse_ensure!(0 (cond $stack) $bail ($($fuel)*) {($($buf)* $while) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($for:tt $($dup:tt)*) for $($rest:tt)*) => {
        $crate::__parse_ensure!(pat (cond $stack) $bail ($($fuel)*) {($($buf)* $for) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom (cond $stack:tt) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($brace:tt $($dup:tt)*) {$($block:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(cond $stack $bail ($($fuel)*) {($($buf)* $brace) $($parse)*} ($($rest)*) $($rest)*)
    };

    (cond $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($else:tt $if:tt $($dup:tt)*) else if $($rest:tt)*) => {
        $crate::__parse_ensure!(0 (cond $stack) $bail ($($fuel)*) {($($buf)* $else $if) $($parse)*} ($($rest)*) $($rest)*)
    };

    (cond $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($else:tt $brace:tt $($dup:tt)*) else {$($block:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $else $brace) $($parse)*} ($($rest)*) $($rest)*)
    };

    (cond $stack:tt $bail:tt (~$($fuel:tt)*) $parse:tt $dup:tt $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) $parse $dup $($rest)*)
    };

    // atomic expressions

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($paren:tt $($dup:tt)*) ($($content:tt)*) $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $paren) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($bracket:tt $($dup:tt)*) [$($array:tt)*] $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $bracket) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($brace:tt $($dup:tt)*) {$($block:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $brace) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($loop:tt $block:tt $($dup:tt)*) loop {$($body:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $loop $block) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($async:tt $block:tt $($dup:tt)*) async {$($body:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $async $block) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($async:tt $move:tt $block:tt $($dup:tt)*) async move {$($body:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $async $move $block) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($unsafe:tt $block:tt $($dup:tt)*) unsafe {$($body:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $unsafe $block) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($const:tt $block:tt $($dup:tt)*) const {$($body:tt)*} $($rest:tt)*) => {
        // TODO: this is mostly useless due to https://github.com/rust-lang/rust/issues/86730
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $const $block) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} $dup:tt $lit:literal $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $lit) $($parse)*} ($($rest)*) $($rest)*)
    };

    // path expressions

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($colons:tt $($dup:tt)*) :: $ident:ident $($rest:tt)*) => {
        $crate::__parse_ensure!(epath (atom $stack) $bail ($($fuel)*) {($($buf)* $colons $ident) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} $dup:tt $ident:ident $($rest:tt)*) => {
        $crate::__parse_ensure!(epath (atom $stack) $bail ($($fuel)*) {($($buf)* $ident) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($langle:tt $($dup:tt)*) < $($rest:tt)*) => {
        $crate::__parse_ensure!(type (qpath (epath (atom $stack))) $bail ($($fuel)*) {($($buf)* $langle) $($parse)*} ($($rest)*) $($rest)*)
    };

    (epath $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($colons:tt $langle:tt $($dup:tt)*) :: < $($rest:tt)*) => {
        $crate::__parse_ensure!(generic (epath $stack) $bail ($($fuel)*) {($($buf)* $colons $langle) $($parse)*} ($($rest)*) $($rest)*)
    };

    (epath $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($colons:tt $($dup:tt)*) :: << $($rest:tt)*) => {
        $crate::__parse_ensure!(generic (epath $stack) $bail ($($fuel)*) {($($buf)* $colons <) $($parse)*} (< $($rest)*) < $($rest)*)
    };

    (epath $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($colons:tt $($dup:tt)*) :: <- $($rest:tt)*) => {
        $crate::__parse_ensure!(generic (epath $stack) $bail ($($fuel)*) {($($buf)* $colons <) $($parse)*} (- $($rest)*) - $($rest)*)
    };

    (epath $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($colons:tt $($dup:tt)*) :: $ident:ident $($rest:tt)*) => {
        $crate::__parse_ensure!(epath $stack $bail ($($fuel)*) {($($buf)* $colons $ident) $($parse)*} ($($rest)*) $($rest)*)
    };

    (epath ($pop:ident $stack:tt) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($bang:tt $args:tt $($dup:tt)*) ! ($($mac:tt)*) $($rest:tt)*) => {
        $crate::__parse_ensure!($pop $stack $bail ($($fuel)*) {($($buf)* $bang $args) $($parse)*} ($($rest)*) $($rest)*)
    };

    (epath ($pop:ident $stack:tt) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($bang:tt $args:tt $($dup:tt)*) ! [$($mac:tt)*] $($rest:tt)*) => {
        $crate::__parse_ensure!($pop $stack $bail ($($fuel)*) {($($buf)* $bang $args) $($parse)*} ($($rest)*) $($rest)*)
    };

    (epath ($pop:ident $stack:tt) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($bang:tt $args:tt $($dup:tt)*) ! {$($mac:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!($pop $stack $bail ($($fuel)*) {($($buf)* $bang $args) $($parse)*} ($($rest)*) $($rest)*)
    };

    (epath ($pop:ident $stack:tt) $bail:tt (~$($fuel:tt)*) $parse:tt $dup:tt $($rest:tt)*) => {
        $crate::__parse_ensure!($pop $stack $bail ($($fuel)*) $parse $dup $($rest)*)
    };

    // trailer expressions

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($paren:tt $($dup:tt)*) ($($call:tt)*) $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $paren) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($bracket:tt $($dup:tt)*) [$($index:tt)*] $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $bracket) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($brace:tt $($dup:tt)*) {$($init:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $brace) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($question:tt $($dup:tt)*) ? $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $question) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($dot:tt $ident:tt $colons:tt $langle:tt $($dup:tt)*) . $i:ident :: < $($rest:tt)*) => {
        $crate::__parse_ensure!(generic (atom $stack) $bail ($($fuel)*) {($($buf)* $dot $ident $colons $langle) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($dot:tt $ident:tt $colons:tt $($dup:tt)*) . $i:ident :: << $($rest:tt)*) => {
        $crate::__parse_ensure!(generic (atom $stack) $bail ($($fuel)*) {($($buf)* $dot $ident $colons <) $($parse)*} (< $($rest)*) < $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($dot:tt $ident:tt $colons:tt $($dup:tt)*) . $i:ident :: <- $($rest:tt)*) => {
        $crate::__parse_ensure!(generic (atom $stack) $bail ($($fuel)*) {($($buf)* $dot $ident $colons <) $($parse)*} (- $($rest)*) - $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($dot:tt $($dup:tt)*) . $field:ident $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $dot $field) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($dot:tt $index:tt $($dup:tt)*) . $lit:literal $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $dot $index) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($as:tt $($dup:tt)*) as $($rest:tt)*) => {
        $crate::__parse_ensure!(type (atom $stack) $bail ($($fuel)*) {($($buf)* $as) $($parse)*} ($($rest)*) $($rest)*)
    };

    // types

    (type ($pop:ident $stack:tt) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($bracket:tt $($dup:tt)*) [$($content:tt)*] $($rest:tt)*) => {
        $crate::__parse_ensure!($pop $stack $bail ($($fuel)*) {($($buf)* $bracket) $($parse)*} ($($rest)*) $($rest)*)
    };

    (type ($pop:ident $stack:tt) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($paren:tt $($dup:tt)*) ($($content:tt)*) $($rest:tt)*) => {
        $crate::__parse_ensure!($pop $stack $bail ($($fuel)*) {($($buf)* $paren) $($parse)*} ($($rest)*) $($rest)*)
    };

    (type $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($star:tt $const:tt $($dup:tt)*) *const $($rest:tt)*) => {
        $crate::__parse_ensure!(type $stack $bail ($($fuel)*) {($($buf)* $star $const) $($parse)*} ($($rest)*) $($rest)*)
    };

    (type $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($star:tt $mut:tt $($dup:tt)*) *mut $($rest:tt)*) => {
        $crate::__parse_ensure!(type $stack $bail ($($fuel)*) {($($buf)* $star $mut) $($parse)*} ($($rest)*) $($rest)*)
    };

    (type $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($and:tt $life:lifetime $mut:tt $($dup:tt)*) & $l:lifetime mut $($rest:tt)*) => {
        $crate::__parse_ensure!(type $stack $bail ($($fuel)*) {($($buf)* $and $life $mut) $($parse)*} ($($rest)*) $($rest)*)