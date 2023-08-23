const { eventHandler } = require("./event");
const { SerialPort } = require("serialport");
const { ReadlineParser } = require("@serialport/parser-readline");
const { listStreamDecks, openStreamDeck } = require("@elgato-stream-deck/node");
const WebSocketServer = require("ws").Server;
const EventEmitter = require("events");
const Jimp = require("jimp");

const store = require("./store");
const { createUniqueId } = require("./shared");

class BaseDevice extends EventEmitter {}

class OceanDeckBaseDevice extends BaseDevice {
	type = 7;
	name = "OceanDeck";

	keys = 9;
	sliders = 2;

	rows = 3;
	columns = 3;

	lastKey = 0;
	lastSliders = [ 0, 0 ];

	handle(data) {
		if (typeof data == "string") data = JSON.parse(data);
		if (data.key <= 0) {
			if (this.lastKey > 0) {
				this.emit("keyUp", this.lastKey - 1);
				this.lastKey = 0;
			}
		} else if (data.key) {
			this.lastKey = data.key;
			this.emit("keyDown", data.key - 1);
		}

		if (data.slider0) {
			this.emit("dialRotate", 0, data.slider0 - this.lastSliders[0]);
			this.lastSliders[0] = data.slider0;
		}
		if (data.slider1) {
			this.emit("dialRotate", 1, data.slider1 - this.lastSliders[1]);
			this.lastSliders[1] = data.slider1;
		}
	}

	setImage() {}
}

class OceanDeckWiredDevice extends OceanDeckBaseDevice {
	constructor(path) {
		super();
		this.path = path;
		try { this.port = new SerialPort({ path: this.path, baudRate: 57600 }); }
		catch { return undefined; }
		this.port.on("close", () => this.emit("disconnect"));

		this.parser = this.port.pipe(new ReadlineParser({ delimiter: "\r\n" }));
		this.parser.on("data", (data) => this.handle(data));
	}
}

class OceanDeckVirtualDevice extends OceanDeckBaseDevice {
	constructor(port) {
		super();
		this.name = "Virtual OceanDeck";
		this.server = new WebSocketServer({ port });
		this.server.once("connection", (ws) => {
			ws.on("message", (data) => this.handle(data));
			ws.on("close", () => this.emit("disconnect"));
		});
	}
}

class ElgatoDevice extends BaseDevice {
	constructor(path) {
		super();
		this.path = path;
		this.device = openStreamDeck(this.path);
		switch (this.device.MODEL) {
			case "original", "originalv2", "original-mk2": this.type = 0; break;
			case "mini", "miniv2": this.type = 1; break;
			case "xl", "xlv2": this.type = 2; break;
			case "pedal": this.type = 5; break;
			case "plus": this.type = 7; break;
		}
		this.name = this.device.PRODUCT_NAME;
		this.keys = this.device.NUM_KEYS;
		this.sliders = 0;
		this.rows = this.device.KEY_ROWS;
		this.columns = this.device.KEY_COLUMNS;

		this.device.on("down", (key) => this.emit("keyDown", this.convertIndex(key)));
		this.device.on("up", (key) => this.emit("keyUp", this.convertIndex(key)));
	}

	convertIndex(index) {
		let m = (index + 1) % this.columns;
		if (m != 0) m = this.columns - m;
		m += (Math.floor(index / 5) * this.columns);
		return m;
	}

	setImage(key, image) {
		let d = image;
		let base64re = /data:image\/(apng|avif|gif|jpeg|png|svg\+xml|webp|bmp|x-icon|tiff);base64,([A-Za-z0-9+/]+={0,2})?/;
		if (base64re.test(image)) {
			d = Buffer.from(base64re.exec(image)[2], "base64");
		}
		Jimp.read(d).then((image) => {
			this.device.fillKeyBuffer(convertIndex(key), image.resize(this.device.ICON_SIZE, this.device.ICON_SIZE).bitmap.data, { format: "rgba" });
		});
	}
}

class DeviceManager {
	constructor() {
		this.lastKey = 0;
		this.lastSliders = [ 0, 0 ];

		this.devices = {};
		
		this.initDevice("od-testdevice1", new OceanDeckVirtualDevice(1925));
		this.initDevice("od-testdevice2", new OceanDeckVirtualDevice(1926));
		
		SerialPort.list().then((ports) => {
			ports.forEach((port) => {
				if (!(port.vendorId == "10c4" && port.productId == "ea60")) return;
				this.initDevice("od-" + port.path, new OceanDeckWiredDevice(port.path));
			});
		});
		listStreamDecks().forEach((device) => this.initDevice("sd-" + device.serialNumber, new ElgatoDevice(device.path)));
	}

	initDevice(id, device) {
		this.devices[id] = device;
		let d = store.get("devices");
		if (!d[id]) {
			let randomDefaultProfileId = createUniqueId();
			d[id] = {
				name: device.name,
				keys: device.keys,
				sliders: device.sliders,
				rows: device.rows,
				columns: device.columns,
				profiles: {
					[randomDefaultProfileId]: {
						name: "Default",
						key: [ [ null ], [ null ], [ null ], [ null ], [ null ], [ null ], [ null ], [ null ], [ null ] ],
						slider: [ [ null ], [ null ] ]
					}
				},
				selectedProfile: randomDefaultProfileId
			}
		}
		eventHandler.deviceDidConnect(id, device);
		device.on("disconnect", () => eventHandler.deviceDidDisconnect(id));
		device.on("keyDown", (key) => eventHandler.keyDown(id, key));
		device.on("keyUp", (key) => eventHandler.keyUp(id, key));
		device.on("dialRotate", (dial, value) => eventHandler.dialRotate(id, dial, value));
		store.set("devices", d);
	}
}

const deviceManager = new DeviceManager();
module.exports = { deviceManager };