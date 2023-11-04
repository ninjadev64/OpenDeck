import { StreamDeck, listStreamDecks, openStreamDeck } from "@elgato-stream-deck/node";
import { Resvg } from "@resvg/resvg-js";
import { ReadlineParser } from "@serialport/parser-readline";
import EventEmitter from "events";
import { XMLParser } from "fast-xml-parser";
import { promises } from "fs";
import Jimp from "jimp";
import { SerialPort } from "serialport";
import { Server as WebSocketServer } from "ws";
import { eventHandler } from "./event";
import { getMainWindow } from "./main";
import { ActionTitle, createUniqueId, currentProfiles, error } from "./shared";
import store from "./store";

export interface Device {
	readonly name: string;
	readonly type: number;

	readonly sliders: number;
	readonly keys: number;
	readonly rows: number;
	readonly columns: number;

	on(event: string, callback: Function): void;
	setImage(key: number, source: string, title: ActionTitle | null): void;
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
		if (data.address) this.emit("register", data.address);
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
	port: SerialPort;
	parser: ReadlineParser;

	constructor(path: string) {
		super();

		try { this.port = new SerialPort({ path: path, baudRate: 57600 }); }
		catch { return undefined; }
		this.port.on("close", () => this.emit("disconnect"));

		this.parser = this.port.pipe(new ReadlineParser({ delimiter: "\r\n" }));
		this.parser.on("data", (data) => this.handle(data));

		this.port.write("register");
	}
}

class ProntoKeyBluetoothDevice extends ProntoKeyBaseDevice {
	address: string;
	port: any;
	buffer: string = "";

	constructor(address: string, port: any) {
		super();

		this.address = address;
		this.port = port;
		this.port.on("data", (buffer: Buffer) => {
			this.buffer += buffer.toString("utf-8");
			if (this.buffer.includes("}")) {
				this.handle(this.buffer.slice(0, this.buffer.indexOf("}") + 1));
				this.buffer = this.buffer.slice(this.buffer.indexOf("}") + 1);
			}
		});

		this.port.on("failure", () => {
			this.port.removeAllListeners("data");
			this.port.removeAllListeners("disconnect");
			this.emit("disconnect");
			this.port.inquire();
		});
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

		this.device.on("down", (key) => this.emit("keyDown", key));
		this.device.on("up", (key) => this.emit("keyUp", key));

		if (this.device.MODEL == "plus") {
			this.sliders = 4;
			this.device.on("encoderDown", (encoder: number) => this.emit("dialDown", encoder));
			this.device.on("encoderUp", (encoder: number) => this.emit("dialUp", encoder));
			this.device.on("rotateLeft", (encoder: number, amount: number) => this.emit("dialRotate", encoder, -amount));
			this.device.on("rotateRight", (encoder: number, amount: number) => this.emit("dialRotate", encoder, amount));
		}
	}

	convertIndex(index: number): number {
		let m = (index + 1) % this.columns;
		if (m != 0) m = this.columns - m;
		m += (Math.floor(index / this.columns) * this.columns);
		return m;
	}

	async setImage(key: number, source: string, title: ActionTitle | null): Promise<void> {
		if (!source) {
			this.device.fillKeyColor(key, 0, 0, 0);
			return;
		}

		let base64re = /^data:image\/(apng|avif|gif|jpeg|png|svg\+xml|webp|bmp|x-icon|tiff);base64,([A-Za-z0-9+/]+={0,2})?/;
		let svgxmlre = /^data:image\/svg\+xml,(.+)/;
		
		let d: Buffer;
		if (base64re.test(source)) d = Buffer.from(base64re.exec(source)[2], "base64");
		else if (svgxmlre.test(source)) d = Buffer.from(svgxmlre.exec(source)[1]);
		else d = await promises.readFile(source);
		
		try {
			if ("svg" in (new XMLParser().parse(d))) {
				d = new Resvg(d, {
					font: { loadSystemFonts: false, fontFiles: [ "../assets/Rubik.ttf" ] },
					fitTo: { mode: "width", value: this.device.ICON_SIZE }
				}).render().asPng();
			}
		} catch (_) {}

		const image = await Jimp.read(d);
		image.resize(this.device.ICON_SIZE, this.device.ICON_SIZE);
		if (title && title.show) {
			image.print(await Jimp.loadFont(Jimp.FONT_SANS_32_WHITE), 0, 0, {
				text: title.text,
				alignmentX: Jimp.HORIZONTAL_ALIGN_CENTER,
				alignmentY: title.alignment == "top" ? Jimp.VERTICAL_ALIGN_TOP : (title.alignment == "bottom" ? Jimp.VERTICAL_ALIGN_BOTTOM : Jimp.VERTICAL_ALIGN_MIDDLE)
			}, image.getWidth(), image.getHeight());
		}
		this.device.fillKeyBuffer(key, image.bitmap.data, { format: "rgba" });
	}
}

class DeviceManager {
	devices: { [id: string]: Device } = {};

	constructor() {
		if (store.get("useBluetoothProntoKey") && require("bluetooth-serial-port")) {
			const { BluetoothSerialPort } = require("bluetooth-serial-port");
			const bt = new BluetoothSerialPort();
			bt.on("found", (address: string, name: string) => {
				if (name != "ProntoKey") return;
				bt.findSerialPortChannel(address, (channel: number) => bt.connect(address, channel,
					() => this.initDevice("pk-" + address, new ProntoKeyBluetoothDevice(address, bt)),
					() => error(`Failed to connect to Bluetooth device with address ${address}`, false)
				));
			});
			bt.on("finished", () => {
				if (!bt.isOpen()) setTimeout(() => bt.inquire(), 10e3);
			});
			bt.inquire();
		} else {
			SerialPort.list().then((ports) => {
				ports.forEach((port) => {
					if (!port.vendorId || !port.productId) return;
					if (port.vendorId.toLowerCase() != "10c4" || port.productId.toLowerCase() != "ea60") return;
					let device = new ProntoKeyWiredDevice(port.path);
					device.once("register", (address: string) => this.initDevice("pk-" + address, device));
				});
			});
		}
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
						key: Array.from({ length: device.keys }, () => [ null ]),
						slider: Array.from({ length: device.sliders }, () => [ null ])
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

		device.on("dialDown", (dial: number) => eventHandler.dialDown(id, dial));
		device.on("dialUp", (dial: number) => eventHandler.dialUp(id, dial));
		device.on("dialRotate", (dial: number, value: number) => eventHandler.dialRotate(id, dial, value));
		
		store.set("devices", d);
		if (getMainWindow()) getMainWindow().webContents.send("devices", store.get("devices"));
		return device;
	}

	setImage(device: string, key: number, source: string, text: ActionTitle | null): void {
		let d = this.devices[device];
		if (!d) return;
		d.setImage(key, source, text);
	}
}

export const deviceManager = new DeviceManager();
