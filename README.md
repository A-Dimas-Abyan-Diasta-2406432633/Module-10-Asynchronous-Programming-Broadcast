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
