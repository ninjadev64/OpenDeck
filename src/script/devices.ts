import { StreamDeck, listStreamDecks, openStreamDeck } from "@elgato-stream-deck/node";
import { ReadlineParser } from "@serialport/parser-readline";
import EventEmitter from "events";
import Jimp from "jimp";
import { SerialPort } from "serialport";
import { Server as WebSocketServer } from "ws";
import { eventHandler } from "./event";

import { getMainWindow } from "./main";
import { createUniqueId, currentProfiles } from "./shared";
import store from "./store";

export interface Device {
	readonly name: string;
	readonly type: number;

	readonly sliders: number;
	readonly keys: number;
	readonly rows: number;
	readonly columns: number;

	on(event: string, callback: Function): void;
	setImage(key: number, image: string): void;
}

class ProntoKeyBaseDevice extends EventEmitter implements Device {
	name = "ProntoKey";
	type = 7;

	sliders = 2;
	keys = 9;
	rows = 3;
	columns = 3;

	lastKey = 0;
	lastSliders = [ 0, 0 ];

	handle(data: any): void {
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

class ProntoKeyWiredDevice extends ProntoKeyBaseDevice {
	path: string;
	port: SerialPort;
	parser: ReadlineParser;

	constructor(path: string) {
		super();
		this.path = path;
		try { this.port = new SerialPort({ path: this.path, baudRate: 57600 }); }
		catch { return undefined; }
		this.port.on("close", () => this.emit("disconnect"));

		this.parser = this.port.pipe(new ReadlineParser({ delimiter: "\r\n" }));
		this.parser.on("data", (data) => this.handle(data));
	}
}

class ProntoKeyVirtualDevice extends ProntoKeyBaseDevice {
	server: WebSocketServer;

	constructor(port: number) {
		super();
		this.name = "Virtual ProntoKey";
		this.server = new WebSocketServer({ port });
		this.server.once("connection", (ws: any) => {
			ws.on("message", (data: string) => this.handle(data));
			ws.on("close", () => this.emit("disconnect"));
		});
	}
}

class ElgatoDevice extends EventEmitter implements Device {
	name: string;
	type: number;

	sliders = 0;
	keys: number;
	rows: number;
	columns: number;
	
	path: string;
	device: StreamDeck;

	constructor(path: string) {
		super();
		this.path = path;
		this.device = openStreamDeck(this.path);
		switch (this.device.MODEL) {
			case "original": case "originalv2": case "original-mk2": this.type = 0; break;
			case "mini": case "miniv2": this.type = 1; break;
			case "xl": case "xlv2": this.type = 2; break;
			case "pedal": this.type = 5; break;
			case "plus": this.type = 7; break;
		}
		this.name = this.device.PRODUCT_NAME;
		this.keys = this.device.NUM_KEYS;
		this.rows = this.device.KEY_ROWS;
		this.columns = this.device.KEY_COLUMNS;

		this.device.on("down", (key) => this.emit("keyDown", this.convertIndex(key)));
		this.device.on("up", (key) => this.emit("keyUp", this.convertIndex(key)));
	}

	convertIndex(index: number): number {
		let m = (index + 1) % this.columns;
		if (m != 0) m = this.columns - m;
		m += (Math.floor(index / 5) * this.columns);
		return m;
	}

	setImage(key: number, image: string): void {
		let d: any = image;
		let base64re = /data:image\/(apng|avif|gif|jpeg|png|svg\+xml|webp|bmp|x-icon|tiff);base64,([A-Za-z0-9+/]+={0,2})?/;
		if (base64re.test(image)) {
			d = Buffer.from(base64re.exec(image)[2], "base64");
		}
		Jimp.read(d).then((image) => {
			this.device.fillKeyBuffer(this.convertIndex(key), image.resize(this.device.ICON_SIZE, this.device.ICON_SIZE).bitmap.data, { format: "rgba" });
		});
	}
}

class DeviceManager {
	devices: { [id: string]: Device };

	constructor() {
		this.devices = {};
		
		SerialPort.list().then((ports) => {
			ports.forEach((port) => {
				if (!(port.vendorId.toLowerCase() == "10c4" && port.productId.toLowerCase() == "ea60")) return;
				this.initDevice("pk-" + port.path, new ProntoKeyWiredDevice(port.path));
			});
			this.initDevice("pk-testdevice1", new ProntoKeyVirtualDevice(1925));
			getMainWindow().webContents.send("devices", store.get("devices"));
		});
		listStreamDecks().forEach((device) => this.initDevice("sd-" + device.serialNumber, new ElgatoDevice(device.path)));
	}

	initDevice(id: string, device: Device): Device {
		this.devices[id] = device;
		let d = store.get("devices");
		if (!d[id]) {
			let randomDefaultProfileId = createUniqueId();
			d[id] = {
				name: device.name,
				type: device.type,
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
			currentProfiles[id] = d[id].profiles[randomDefaultProfileId];
		}
		eventHandler.deviceDidConnect(id, device);
		device.on("disconnect", () => eventHandler.deviceDidDisconnect(id));
		device.on("keyDown", (key: number) => eventHandler.keyDown(id, key));
		device.on("keyUp", (key: number) => eventHandler.keyUp(id, key));
		device.on("dialRotate", (dial: number, value: number) => eventHandler.dialRotate(id, dial, value));
		store.set("devices", d);
		return device;
	}
}

export const deviceManager = new DeviceManager();