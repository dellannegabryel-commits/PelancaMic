## 📱 Cliente (Mobile) — `Docs/mobile/README.md`

**Módulo:** Aplicativo Android (Kotlin + Jetpack Compose)
**Objetivo:** Capturar áudio com prioridade de hardware, encapsular no protocolo binário do PelancaMic e transmitir ininterruptamente, oferecendo uma UI de controle com zero travamentos.

### Fluxo de Captura e Transporte

* **Captura (AudioRecord):**
* Configurado nativamente para 48.000 Hz, Canal Mono, 16-bit PCM.
* Lê blocos exatos de 10ms (480 amostras / 960 bytes).


* **Protocolo de Áudio (Header 16-bytes):**
Antes de enviar ao PC via `DatagramSocket`, o Kotlin anexa o seguinte cabeçalho (Big-Endian) a cada bloco:
* `0-1`: Magic `0x4D53`
* `2`: Version `1`
* `3`: Flags (Muted / Silence)
* `4-7`: Sequence (UInt32 incrementado)
* `8-11`: Timestamp de captura (UInt32, para cálculo de jitter)
* `12-13`: Payload Length (geralmente 960 bytes)
* `14`: Channels (`1`)
* `15`: Codec (`0` para PCM)


* **Descoberta e Conexão:**
Ao abrir, o app escuta a porta UDP 41234. Ao receber o beacon do PC, exibe o computador na tela. Ao tocar, abre o WebSocket na 41236 para sinalização de estado.

### Arquitetura de Interface (UI/UX)

* **Jetpack Compose:** Toda a interface é declarativa.
* **Estética:** Material You (cores extraídas do papel de parede via `dynamicColorScheme`), layouts em "Squircle" (CircleShape) e comportamento Edge-to-Edge.
* **Glassmorphism:** Utilização da biblioteca Haze (`dev.chrisbanes.haze`) para criar painéis de status translúcidos com desfoque de fundo (Backdrop Blur).
* **Animação:** Botão central de Mute reage com física de mola (`spring`) para simular o afundamento de hardware real.

---
