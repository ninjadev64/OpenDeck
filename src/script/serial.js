const { eventHandler } = require("./event");
const { SerialPort } = require("serialport");
const { ReadlineParser } = require("@serialport/parser-readline");

const WebSocketServer = require("ws").Server;

const Store = require("electron-store");
const store = new Store();
SerialPort.list().then((ports) => { store.set("allPorts", ports); });

class SerialInterface {
    constructor(ws) {
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
                eventHandler.keyUp(this.lastPressed);
                this.lastPressed = 0;
            }
            return;
        }

        this.lastPressed = data.button;
        eventHandler.keyDown(data.button);
    }
}

const serialInterface = new SerialInterface(true);
module.exports = { serialInterface };