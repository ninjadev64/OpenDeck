const RPC = require("discord-rpc");

class DiscordActions {
    constructor() {
        const clientId = "1023592341484863638";
        const scopes = [ "rpc", "rpc.voice.read", "rpc.voice.write" ];
        this.client = new RPC.Client({ transport: 'ipc' });

        (async () => {
            let token = localStorage.getItem("discordToken");
            if (token) {
                this.client.login({ clientId, clientSecret, accessToken: localStorage.getItem("discordToken"), scopes, redirectUri: 'http://localhost:53134' });
            } else {
                this.client.login({ clientId, clientSecret, scopes, redirectUri: 'http://localhost:53134' });
            }

            this.client.on('ready', () => {
                console.log('Authed for Discord user', this.client?.user?.username);
                localStorage.setItem("discordToken", this.client?.accessToken);
            });
        })();
    }

    mute(_) {
        this.client.getVoiceSettings().then((settings) => {
            this.client.setVoiceSettings({
                mute: !settings.mute
            });
        });
    }
    deafen(_) {
        this.client.getVoiceSettings().then((settings) => {
            this.client.setVoiceSettings({
                deaf: !settings.deaf
            });
        });
    }
}