const { pluginManager } = require("./plugins");
const { SerialPort } = require("serialport");
const { ReadlineParser } = require("@serialport/parser-readline");

const WebSocketServer = require("ws").Server;

const Store = require("electron-store");
const store = new Store();
SerialPort.list().then((ports) => { store.set("allPorts", ports); });

class SerialInterface {
    constructor(ws) {
        this.buttons = {
            0: { plugin: "com.elgato.template.sdPlugin", action: "com.elgato.template.action" },
            1: { plugin: "com.elgato.template.sdPlugin", action: "com.elgato.template.action" },
            2: { plugin: "com.elgato.template.sdPlugin", action: "com.elgato.template.action" },
            3: { plugin: "com.elgato.template.sdPlugin", action: "com.elgato.template.action" },
            4: { plugin: "com.elgato.template.sdPlugin", action: "com.elgato.template.action" },
            5: { plugin: "com.elgato.template.sdPlugin", action: "com.elgato.template.action" },
            6: { plugin: "com.elgato.template.sdPlugin", action: "com.elgato.template.action" },
            7: { plugin: "com.elgato.template.sdPlugin", action: "com.elgato.template.action" },
            8: { plugin: "com.elgato.template.sdPlugin", action: "com.elgato.template.action" },
            9: { plugin: "com.elgato.template.sdPlugin", action: "com.elgato.template.action" }
        }

        this.lastPressed = 0;

        if (ws) { // Mock an OceanDeck over a WebSocket connection
            this.server = new WebSocketServer({ port: 1925 });
            this.server.on("connection", (ws) => {
                ws.on("message", (data) => {
                    this.handle(JSON.parse(data));
                });
            });
        } else { // Use serial as normal
            this.port = new SerialPort({ path: store.get("serialPort"), baudRate: 57600 });
            this.parser = this.port.pipe(new ReadlineParser({ delimiter: "\r\n" }));

            this.parser.on("data", (data) => {
                this.handle(JSON.parse(data));
            });
        }
    }

    handle(data) {
        if (data.button <= 0) {
            if (this.lastPressed > 0) {
                pluginManager.sendEvent(this.buttons[this.lastPressed].plugin, 
                    {
                        event: "keyUp",
                        action: this.buttons[this.lastPressed].action,
                        context: this.lastPressed,
                        device: 0,
                        payload: {
                            settings: {},
                            coordinates: {
                                row: Math.floor(this.lastPressed / 3) + 1,
                                column: this.lastPressed % 3
                            },
                            isInMultiAction: false
                        }
                    }
                );
                this.lastPressed = 0;
            }
            return;
        }

        this.lastPressed = data.button;
        pluginManager.sendEvent(this.buttons[data.button].plugin, 
            {
                event: "keyDown",
                action: this.buttons[data.button].action,
                context: data.button,
                device: 0,
                payload: {
                    settings: {},
                    coordinates: {
                        row: Math.floor(data.button / 3) + 1,
                        column: data.button % 3
                    },
                    isInMultiAction: false
                }
            }
        );
    }
}

const serialInterface = new SerialInterface(true);
module.exports = { serialInterface };