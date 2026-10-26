# PelancaMic 🎙️

<div align="center">
  <p><strong>Transforme seu smartphone Android em um microfone de estúdio sem fio com latência zero.</strong></p>
</div>

O **PelancaMic** é um aplicativo de código aberto construído com foco agressivo em performance. Ele utiliza comunicação P2P em rede local (LAN) para enviar áudio não compactado do seu celular diretamente para um cabo de áudio virtual no PC, sendo instantaneamente reconhecido por jogos, Discord, OBS e softwares de reunião.

## 🚀 Por que o PelancaMic?

O projeto foi desenhado para contornar as limitações de frameworks híbridos, utilizando tecnologias nativas e de baixo nível para extrair cada milissegundo de performance da sua rede:

- **Mobile (Cliente):** Escrito 100% em **Kotlin + Jetpack Compose**. Uso direto da API `AudioRecord` para captura de áudio com prioridade máxima do sistema Android.
- **Desktop (Servidor):** Construído com **Tauri v3 e Rust**. Processamento de pacotes UDP, Jitter Buffer adaptativo e injeção de áudio diretamente no hardware, garantindo consumo quase nulo de RAM e CPU enquanto você joga.

## 📐 Topologia de Rede

O sistema se comunica de forma descentralizada na sua rede Wi-Fi utilizando três canais independentes:

| Serviço | Protocolo | Porta | Papel |
| :--- | :--- | :--- | :--- |
| **Descoberta** | UDP Broadcast | `41234` | O PC anuncia sua presença. O Android escuta e conecta (Zero Config). |
| **Áudio** | UDP Unicast | `41235` | Streaming unidirecional (Frames PCM 16-bit de 10ms com cabeçalho de 16-bytes). |
| **Controle** | WebSocket (TCP) | `41236` | Sincronização de estado (Mute, Volume, Ping/Pong e Qualidade da rede). |

## 📂 Estrutura do Monorepo

```text
pelancamic/
├── mobile/                  # App Android nativo (Kotlin + Compose)
│   ├── app/src/main/java/   # Lógica de Captura e Sockets
│   └── app/src/main/res/    # UI Material 3 / Glassmorphism
└── pc/                      # Servidor Desktop (Tauri + Rust)
    ├── src/                 # Frontend (React 18 + Vite)
    └── src-tauri/src/       # Backend Nativo (Rust, cpal, tokio)
