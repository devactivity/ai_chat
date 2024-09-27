# Ollama RAG Documents

## Usage

Silakan buat _python project env_ dan install dependency terlebih dahulu

```bash
python -m venv .venv
source ./venv/bin/activate

pip install -r requirements.txt
pip install "fastapi[standard]" llama-index-vector-stores-chroma
```

Pastikan service `Ollama` sudah berjalan, kemudian jalankan aplikasi dengan perintah:

```bash
fastapi dev main.py
```

Pada aplikasi ChatTUI, silahkan ganti _endpoint_ ke _host_ aplikasi ini, dengan begitu aplikasi ChatTUI tetap berfungsi sebagaimana mestinya.

**NOTES**: Response dari RAG ini belum sempurna, _text_ yg dihasilkan masih belum natural namun sudah bisa merujuk ke data dalam direktori `docs` yg dapat anda tambahkan dokumen lain didalamnya.
