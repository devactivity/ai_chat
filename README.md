# ai_chat

Aplikasi di branch `scrolling` ini dijalankan dengan [Ollama](https://ollama.com/), pastikan platform tersebut dijalankan terlebih dahulu.

Untuk _model_ bebas ditentukan sendiri, saat aplikasi ini pertama kali dijalankan, akan membuat _config file_ dengan _default value_ kosong yang berlokasi di `$HOME/.config/chat_tui/config.toml`, atau juga bisa terlebih dahulu sediakan file tersebut secara manual dilokasi yang sama. Berikut isi contoh config yang digunakan:

```toml
api_endpoint = "http://192.168.18.188:11434/api/chat"
model = "MarcoAland/llama3.1-rag-indo"
temperature = 0.7
```

tambahkan payload `url` untuk endpoint `/api/scrape`
