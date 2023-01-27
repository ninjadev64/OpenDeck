const { eventHandler } = require("./event");
const { SerialPort } = require("serialport");
const { ReadlineParser } = require("@serialport/parser-readline");

const store = require("./store");
const WebSocketServer = require("ws").Server;

SerialPort.list().then((ports) => { store.set("allPorts", ports); });

class SerialInterface {
	constructor(ws) {
		this.lastPressed = 0;

		if (ws) { // Mock an OceanDeck over a WebSocket connection
			this.server = new WebSocketServer({ port: 1925 });
			this.server.once("connection", (ws) => {
				eventHandler.deviceDidConnect();
				ws.on("message", (data) => {
					this.handle(JSON.parse(data));
				});
				ws.on("close", eventHandler.deviceDidDisconnect);
			});
		} else { // Use serial as normal
			this.port = new SerialPort({ path: store.get("serialPort"), baudRate: 57600 });
			this.parser = this.port.pipe(new ReadlineParser({ delimiter: "\r\n" }));
			eventHandler.deviceDidConnect();
			this.parser.on("data", (data) => {
				this.handle(JSON.parse(data));
			});
			this.port.on("close", eventHandler.deviceDidDisconnect);
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