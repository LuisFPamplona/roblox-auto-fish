# Build Windows x64

Use Node.js 24 LTS, Rust stable MSVC x64, Microsoft Visual C++ Build Tools (Desktop development with C++) e Windows SDK. O WebView2 Runtime é necessário para executar a interface. Use somente instaladores oficiais. A instalação de Build Tools altera o sistema e requer autorização do proprietário.

Abra Developer PowerShell para x64; confirme `Get-Command cl,link,rc` e `rustc -Vv`. A sessão deve ter PATH, INCLUDE, LIB e LIBPATH definidos pelo ambiente oficial Visual Studio. Não use apenas um `link.exe` avulso.

```powershell
npm ci --ignore-scripts
npm run build
npm run test:security
cd src-tauri
cargo check --locked
cargo test --locked
cargo clippy --locked --all-targets
cargo fmt --all -- --check
cd ..
npm run app:build -- --no-bundle -- --locked
```

O build portátil incorpora os arquivos frontend no executável. Não requer NSIS, instalação do aplicativo, privilégios de administrador ou atualizador. O WebView2 usa processos próprios para renderização. A configuração e os logs ficam em `%APPDATA%\autofish-local`.

O teste WinRT sintético é separado dos testes padrão: configure `AUTOFISH_OCR_FIXTURE` com uma imagem local criada para teste contendo `A Devil Fruit has spawned`, e execute `cargo test --locked --test windows_native winrt_recognizes_synthetic_text_on_independent_threads -- --ignored --nocapture`. Isso não valida OCR dentro do GPO.

O teste replay só valida quadros reais se houver PNGs em `src-tauri/tests/fixtures/reels`. Um retorno sem fixtures não prova reconhecimento de pesca real. Consulte `WINDOWS_VALIDATION_REPORT.md` para resultados e limitações desta máquina.
