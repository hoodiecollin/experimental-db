use crate::{book::Book, BookId, Key, DATA_DIR};

#[test]
fn test_create_book() -> anyhow::Result<()> {
    std::fs::remove_dir_all(DATA_DIR.join("books/0")).ok();

    let book: Book<u8> = Book::new(BookId::new(0)).unwrap();
    let mut book_guard = book.write();
    assert_eq!(book_guard.len(), 0);

    for i in 0..4 {
        let key = Key::rand();
        eprintln!("inserting: {} @ {}", i, key);
        book_guard.insert(Key::rand(), i)?;
    }

    assert_eq!(book_guard.len(), 4usize);

    eprintln!("book: {:#?}", book_guard);

    Ok(())
}
