const MessageType = {
    CHAT: "chat",
    HELP: "help",
    QUIT: "quit",
    PRIVATE: "private",
    NICK: "nick",
};

document.addEventListener("DOMContentLoaded", () => {
    const socket = new WebSocket("ws://localhost:6969");
    const chatMessages = document.querySelector(".chat-messages");
    const messageInput = document.getElementById("messageInput");
    const sendMessageButton = document.getElementById("sendMessage");

    const show_message = (text, type) => {
        const messageElement = document.createElement("div");
        messageElement.className = `message ${type}`;

        // Escape HTML and handle whitespace
        const escapedText = text
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#039;");

        const formattedText = escapedText
            .replace(/\t/g, "&nbsp;&nbsp;&nbsp;&nbsp;")
            .replace(/  /g, "&nbsp;&nbsp;")
            .replace(/\n/g, "<br>");

        messageElement.innerHTML = formattedText;
        chatMessages.appendChild(messageElement);
        chatMessages.scrollTop = chatMessages.scrollHeight;
    };

    const send_message = (type, content) => {
        if (socket.readyState === WebSocket.OPEN) {
            switch (type) {
                case MessageType.CHAT:
                    socket.send(JSON.stringify({ message_type: type, message: content }));
                    break;
                case MessageType.HELP:
                    socket.send(JSON.stringify({ message_type: type }));
                    break;
                case MessageType.QUIT:
                    socket.send(JSON.stringify({ message_type: type }));
                    break;
                case MessageType.PRIVATE:
                    let [nick, ...args] = content.split(' ');
                    message = args.join(' ');
                    socket.send(JSON.stringify({ message_type: type, receiver: nick, message: message }));
                    break;
                case MessageType.NICK:
                    socket.send(JSON.stringify({message_type: type, nick: content}));
                    break;
            }
        } else {
            show_message("WebSocket is not open.", "server");
        }
    };

    socket.onopen = () => {
        show_message("Connected to the chat server.", "server");
    };

    socket.onmessage = (event) => {
        console.log(event.data);
        show_message(event.data, "server");
    };

    socket.onerror = (error) => {
        console.error("WebSocket error:", error);
        show_message("An error occurred with the connection.", "server");
    };

    socket.onclose = (_event) => {
        show_message("Chat connection closed.", "server");
    };

    sendMessageButton.addEventListener("click", () => {
        const message = messageInput.value.trim();
        if (message) {
            if (message.startsWith("/")) {
                const after_prefix = message.substring(1);
                let [command, ...args] = after_prefix.split(' ');
                args = args.join(' ');
                console.log(`args = ${args}`);

                if (command === "quit") {
                    console.log("Command quit");
                    send_message(MessageType.QUIT, "");
                } else if (command === "help") {
                    console.log("Command help");
                    send_message(MessageType.HELP, "");
                } else if (command === "nick") {
                    console.log("Command nick");
                    send_message(MessageType.NICK, args);
                } else if (command === "private") {
                    console.log("Command private");
                    send_message(MessageType.PRIVATE, args);
                } else {
                    console.log("Unknown command");
                    show_message(`Unknown command: ${command}`, "server");
                }
            } else {
                console.log("Normal message");
                send_message(MessageType.CHAT, message);
                show_message(message, "user");
            }

        }
        messageInput.value = ""; // Clear the input
    });

    messageInput.addEventListener("keydown", (event) => {
        if (event.key === "Enter") {
            sendMessageButton.click();
        }
    });
});
