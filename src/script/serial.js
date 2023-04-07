const { eventHandler } = require("./event");
const { SerialPort } = require("serialport");
const { ReadlineParser } = require("@serialport/parser-readline");

const store = require("./store");
const WebSocketServer = require("ws").Server;

SerialPort.list().then((ports) => { store.set("allPorts", ports); });

class SerialInterface {
	constructor(ws) {
		this.lastKey = 0;
		this.lastSliders = [ 0, 0 ];

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
		if (data.key <= 0) {
			if (this.lastKey > 0) {
				eventHandler.keyUp(this.lastKey);
				this.lastKey = 0;
			}
		} else {
			this.lastKey = data.key;
			eventHandler.keyDown(data.key);
		}

		if (data.slider0) {
			eventHandler.dialRotate(0, data.slider0 - this.lastSliders[0]);
			this.lastSliders[0] = data.slider0;
		}
		if (data.slider1) {
			eventHandler.dialRotate(1, data.slider1 - this.lastSliders[1]);
			this.lastSliders[1] = data.slider1;
		}
	}
}

const serialInterface = new SerialInterface(false);
module.exports = { serialInterface };