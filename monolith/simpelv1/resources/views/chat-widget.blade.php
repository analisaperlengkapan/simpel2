@if(config('app.chat_widget_enabled'))
<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Chat Widget Melayang</title>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.0/css/all.min.css">
    <meta name="csrf-token" content="{{ csrf_token() }}">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
        }
        body {
            background: #fff7e0;
            min-height: 100vh;
            padding: 20px;
        }
        /* Chat Widget Styles */
        #chat-toggle {
            position: fixed;
            bottom: 25px;
            right: 25px;
            z-index: 9998;
            background: linear-gradient(135deg, #1e5631 0%, #f9b233 100%);
            color: white;
            width: 60px;
            height: 60px;
            border-radius: 50%;
            border: none;
            font-size: 24px;
            cursor: pointer;
            box-shadow: 0 6px 15px rgba(30, 86, 49, 0.25);
            display: flex;
            justify-content: center;
            align-items: center;
            transition: transform 0.3s, box-shadow 0.3s;
        }
        #chat-toggle:hover {
            transform: scale(1.1);
            box-shadow: 0 8px 20px #f9b23399;
            background: linear-gradient(135deg, #f9b233 0%, #1e5631 100%);
            color: #1e5631;
        }
        #chat-widget {
            position: fixed;
            bottom: 100px;
            right: 25px;
            width: 250px;
            height: 380px;
            background: #fff;
            border-radius: 22px;
            box-shadow: 0 10px 30px #1e563122;
            display: none;
            flex-direction: column;
            z-index: 9999;
            overflow: hidden;
            transition: all 0.3s cubic-bezier(.4,2,.6,1);
            border: 2.5px solid #f9b233;
        }
        #chat-header {
            background: #1e5631;
            color: #fff;
            padding: 15px 20px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .chat-title {
            display: flex;
            align-items: center;
            gap: 10px;
        }
        .chat-title i {
            font-size: 20px;
            color: #f9b233;
        }
        .chat-title img {
            width: 28px;
            height: 28px;
            border-radius: 50%;
            background: #fff7e0;
            border: 1.5px solid #f9b233;
        }
        .chat-controls {
            display: flex;
            gap: 12px;
        }
        .chat-controls button {
            background: rgba(255, 255, 255, 0.2);
            border: none;
            width: 28px;
            height: 28px;
            border-radius: 50%;
            color: #fff;
            cursor: pointer;
            display: flex;
            justify-content: center;
            align-items: center;
            transition: background 0.3s;
        }
        .chat-controls button:hover {
            background: #f9b233;
            color: #1e5631;
        }
        #chat-body {
            padding: 15px;
            overflow-y: auto;
            flex-grow: 1;
            display: flex;
            flex-direction: column;
            gap: 12px;
            background: #fff7e0;
        }
        .chat-bubble {
            max-width: 85%;
            padding: 12px 16px;
            border-radius: 18px;
            position: relative;
            animation: fadeIn 0.3s ease;
            box-shadow: 0 2px 5px #1e56311a;
        }
        @keyframes fadeIn {
            from { opacity: 0; transform: translateY(10px); }
            to { opacity: 1; transform: translateY(0); }
        }
        .chat-bubble.user {
            background: #f9b233;
            color: #1e5631;
            align-self: flex-end;
            border-bottom-right-radius: 5px;
        }
        .chat-bubble.ai {
            background: #fff;
            border: 1.5px solid #1e5631;
            color: #1e5631;
            align-self: flex-start;
            border-bottom-left-radius: 5px;
        }
        .sender-name {
            font-weight: 600;
            font-size: 12px;
            margin-bottom: 5px;
            color: #1e5631;
        }
        .ai .sender-name {
            color: #f9b233;
        }
        .message {
            font-size: 14px;
            line-height: 1.5;
            white-space: pre-wrap;
        }
        .timestamp {
            font-size: 10px;
            text-align: right;
            margin-top: 5px;
            color: #7f8c8d;
            opacity: 0.7;
        }
        #chat-footer {
            padding: 15px;
            border-top: 1.5px solid #f9b233;
            background: #fff;
        }
        #chat-form {
            display: flex;
            gap: 10px;
        }
        #prompt {
            flex-grow: 1;
            padding: 12px 15px;
            border: 1.5px solid #f9b233;
            border-radius: 25px;
            outline: none;
            font-size: 14px;
            transition: border 0.3s;
            background: #fff7e0;
            color: #1e5631;
        }
        #prompt:focus {
            border-color: #1e5631;
        }
        #chat-form button {
            background: #1e5631;
            color: #fff;
            border: none;
            width: 45px;
            height: 45px;
            border-radius: 50%;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 20px;
            transition: background 0.2s, color 0.2s;
        }
        #chat-form button:hover {
            background: #f9b233;
            color: #1e5631;
        }
        /* INDIKATOR MENGETIK BARU - DIPINDAH DI BAWAH PESAN USER */
        .typing-indicator {
            display: none;
            background: #e3f2fd;
            border: 1px solid #bbdefb;
            padding: 10px 15px;
            border-radius: 18px;
            align-self: flex-end;
            margin-top: -8px;
            margin-bottom: 5px;
        }

        .typing-indicator span {
            height: 8px;
            width: 8px;
            background: #2d6cdf;
            border-radius: 50%;
            display: inline-block;
            margin: 0 2px;
            animation: bounce 1.3s infinite;
        }

        .typing-indicator span:nth-child(2) {
            animation-delay: 0.2s;
        }

        .typing-indicator span:nth-child(3) {
            animation-delay: 0.4s;
        }

        @keyframes bounce {
            0%, 100% { transform: translateY(0); }
            50% { transform: translateY(-5px); }
        }

        /* Responsive */
        @media (max-width: 500px) {
            #chat-widget {
                width: 96vw;
                right: 2vw;
                height: 55vh;
                min-width: 0;
            }
            #chat-toggle {
                right: 1vw;
            }
        }
    </style>
</head>
<body>
    <!-- Chat Widget -->
    <div id="chat-widget">
        <div id="chat-header">
            <div class="chat-title">
                <img src="/assets/images/logo_kejaksaan.png" alt="Kejaksaan" style="width:28px;height:28px;vertical-align:middle;"> 
                <span style="font-weight:bold;letter-spacing:1px;">AI Chat SIMPEL</span>
            </div>
            <!-- chat-controls dihilangkan -->
        </div>
        <div id="chat-body">
            <div class="chat-bubble ai">
                <div class="sender-name">Asisten AI</div>
                <div class="message">Halo! Saya Asisten AI. Ada yang bisa saya bantu hari ini?</div>
                <!-- Timestamp akan diisi oleh JavaScript -->
                <div class="timestamp" id="initial-timestamp"></div>
            </div>
            <!-- Indikator typing dihapus dari sini -->
        </div>
        <div id="chat-footer">
            <form id="chat-form">
                <input type="text" id="prompt" placeholder="Tulis pesan..." autocomplete="off">
                <button type="submit"><i class="fas fa-paper-plane"></i></button>
            </form>
        </div>
    </div>

    <!-- Tombol buka/tutup chat -->
    <button id="chat-toggle">
        <i class="fas fa-comment"></i>
    </button>

    <script src="https://cdn.jsdelivr.net/npm/axios/dist/axios.min.js"></script>
    <script>
        // Set CSRF token untuk Axios
        axios.defaults.headers.common['X-CSRF-TOKEN'] = document.querySelector('meta[name="csrf-token"]').getAttribute('content');

        // Fungsi untuk format waktu
        function getCurrentTime() {
            const now = new Date();
            return `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}`;
        }

        // Set waktu untuk pesan awal
        document.getElementById('initial-timestamp').textContent = getCurrentTime();

        // Toggle widget
        document.getElementById('chat-toggle').addEventListener('click', () => {
            const chat = document.getElementById('chat-widget');
            if (chat.style.display === 'flex') {
                chat.style.opacity = 0;
                setTimeout(() => { chat.style.display = 'none'; }, 300);
            } else {
                chat.style.display = 'flex';
                chat.style.opacity = 0;
                setTimeout(() => { chat.style.opacity = 1; }, 10);
                document.getElementById('chat-body').scrollTop = document.getElementById('chat-body').scrollHeight;
            }
        });

        // (Tombol close dan minimize dihilangkan)

        // Form submit
        document.getElementById('chat-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const promptInput = document.getElementById('prompt');
            const prompt = promptInput.value.trim();
            if (!prompt) return;

            // Tambahkan pesan user
            const chatBody = document.getElementById('chat-body');
            const userBubble = document.createElement('div');
            userBubble.className = 'chat-bubble user';
            userBubble.innerHTML = `
                <div class="sender-name">Anda</div>
                <div class="message">${prompt}</div>
                <div class="timestamp">${getCurrentTime()}</div>
            `;
            chatBody.appendChild(userBubble);
            promptInput.value = '';
            // Indikator mengetik
            const typingIndicator = document.createElement('div');
            typingIndicator.className = 'typing-indicator';
            typingIndicator.id = 'user-typing';
            typingIndicator.innerHTML = `<span></span><span></span><span></span>`;
            chatBody.appendChild(typingIndicator);
            typingIndicator.style.display = 'flex';
            chatBody.scrollTop = chatBody.scrollHeight;
            console.log('[ChatWidget] Sending prompt:', prompt);
            try {
                const response = await axios.post('/ai/chat', { prompt });
                console.log('[ChatWidget] Response:', response);
                typingIndicator.remove();
                let aiResponse = 'Maaf, terjadi kesalahan.';
                if (response.data && response.data.response) {
                    aiResponse = response.data.response;
                } else if (response.data && response.data.answer) {
                    aiResponse = response.data.answer;
                } else if (response.data && response.data.message) {
                    aiResponse = response.data.message;
                }
                const aiBubble = document.createElement('div');
                aiBubble.className = 'chat-bubble ai';
                aiBubble.innerHTML = `
                    <div class="sender-name">Asisten AI</div>
                    <div class="message">${aiResponse}</div>
                    <div class="timestamp">${getCurrentTime()}</div>
                `;
                chatBody.appendChild(aiBubble);
                chatBody.scrollTop = chatBody.scrollHeight;
            } catch (error) {
                typingIndicator.remove();
                const errorBubble = document.createElement('div');
                errorBubble.className = 'chat-bubble ai';
                errorBubble.innerHTML = `
                    <div class="sender-name">Asisten AI</div>
                    <div class="message">Maaf, terjadi kesalahan saat memproses permintaan Anda. Silakan coba lagi.</div>
                    <div class="timestamp">${getCurrentTime()}</div>
                `;
                chatBody.appendChild(errorBubble);
                chatBody.scrollTop = chatBody.scrollHeight;
                console.error('[ChatWidget] Error:', error, error?.response?.data);
            }
        });

        // Auto focus pada input ketika widget dibuka
        document.addEventListener('click', (e) => {
            if (e.target.closest('#chat-widget') && document.getElementById('chat-widget').style.display === 'flex') {
                document.getElementById('prompt').focus();
            }
        });
    </script>
</body>
</html>
@endif