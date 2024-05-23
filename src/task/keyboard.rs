use crate::{print, println};
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{task::AtomicWaker, Stream, StreamExt as _};
use pc_keyboard::{layouts, DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1};

static WAKER: AtomicWaker = AtomicWaker::new();

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if queue.push(scancode).is_ok() {
            WAKER.wake();
        } else {
            println!("WARNING: scancode queue full; dropping keyboard input");
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();

    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::RawKey(KeyCode::Backspace) => {
                        print!("\x08");
                    }
                    DecodedKey::RawKey(KeyCode::Tab) => {
                        print!("\t");
                    }
                    DecodedKey::RawKey(KeyCode::Return) => {
                        print!("\n");
                    }

                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(_key) => {}
                }
            }
        }
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    /// Create a new [`ScancodeStream`].
    ///
    /// # Panics
    ///
    /// Panics if called more than once.
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");

        Self { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        // fast path
        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(cx.waker());

        queue.pop().map_or(Poll::Pending, |scancode| {
            WAKER.take();
            Poll::Ready(Some(scancode))
        })
    }
}

impl Default for ScancodeStream {
    fn default() -> Self {
        Self::new()
    }
}
