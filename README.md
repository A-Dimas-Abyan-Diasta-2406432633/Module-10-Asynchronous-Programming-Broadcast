# Module 10 - Asynchronous Programming

## Tutorial 1: Timer

### 1.1 Initial Code

Bagian ini memakai contoh resmi dari Async Book:
- https://rust-lang.github.io/async-book/02_execution/04_executor.html

Struktur kode:
- `tutorial-1-timer/timer_future`: implementasi `TimerFuture`
- `tutorial-1-timer/executor`: implementasi executor sederhana + spawner

Perubahan kecil yang saya lakukan hanya pada teks output agar sesuai signature pribadi:
- `Dimas Komputer: howdy!`
- `Dimas Komputer: done!`

Cara run:

```bash
cd tutorial-1-timer/executor
cargo run
```

Hasil run menunjukkan pesan awal, tunggu sekitar 2 detik, lalu pesan selesai.

### 1.2 Understanding how it works

Saya menambah satu `println!` tepat setelah `spawner.spawn(...)`:

```rust
println!("Dimas Komputer: task sudah di-spawn, executor belum jalan.");
```

Intinya:
- `spawner.spawn(...)` hanya memasukkan task ke queue.
- Task async belum dipoll waktu itu.
- Task baru benar-benar jalan saat `executor.run()` dipanggil.

Jadi urutan output jadi seperti ini:
1. `task sudah di-spawn, executor belum jalan.`
2. `howdy!`
3. tunggu 2 detik
4. `done!`

Screenshot hasil run saya taruh di folder `images/`:
- `images/experiment-1-2-run.png`
