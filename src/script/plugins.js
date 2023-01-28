const os = require("os");
const fs = require("fs");
const path = require("path");
const store = require("./store");
const WebSocketServer = require("ws").Server;

const { allActions, categories, Action } = require("./shared");

const { app, BrowserWindow } = require("electron");
const { spawn } = require("child_process");
const { version } = require("../../package.json");

class StreamDeckPlugin {
	constructor(root, uuid) {
		let manifest = JSON.parse(fs.readFileSync(path.join(root, uuid, "manifest.json"), "utf8"));

		this.uuid = uuid;
		this.name = manifest.Name;
		this.description = manifest.Description;
		this.author = manifest.Author;
		this.version = manifest.Version;
		this.website = manifest.URL;
		this.iconPath = manifest.Icon;
		this.category = manifest.Category || "Custom";
		this.actions = [];
		this.socket = null;
		
		if (categories[this.category] == undefined) categories[this.category] = [];
		manifest.Actions.forEach((action) => {
			let i = new Action(action.Name, action.UUID, this.uuid, action.Tooltip, path.join(root, uuid, action.Icon + ".png"));
			this.actions.push(i);
			allActions[i.uuid] = i;
			categories[this.category].push(i);
		});

		const info = {
			"application": {
				"font": "Rubik",
				"language": "en",
				"platform": (
					os.platform() == "win32" ? "windows" :
					(os.platform() == "darwin" ? "mac" :
					(os.platform() == "linux" ? "linux" : "unknown"))
				),
				"platformVersion": os.version(),
				"version": version
			},
			"plugin": {
				"uuid": this.uuid,
				"version": this.version
			},
			"devicePixelRatio": 0,
			"colors": {
				"buttonPressedBackgroundColor": "#000000", 
				"buttonPressedBorderColor": "#000000", 
				"buttonPressedTextColor": "#000000", 
				"disabledColor": "#000000", 
				"highlightColor": "#000000", 
				"mouseDownColor": "#000000"
			},
			"devices": [
				{
					"id": "OceanDeck",
					"name": "OceanDeck",
					"size": {
						"columns": 3,
						"rows": 3
					},
					"type": 7
				}
			]
		}

		let codePath;
		switch (os.platform()) {
			case "win32":
				manifest.CodePathWin && (codePath = manifest.CodePathWin); break;
			case "darwin":
				manifest.CodePathMac && (codePath = manifest.CodePathMac); break;
			case "linux":
				manifest.CodePathLin && (codePath = manifest.CodePathLin); break;
		}
		if (codePath == undefined) codePath = manifest.CodePath;
		
		if (codePath.endsWith(".html")) {
			this.window = new BrowserWindow({
				autoHideMenuBar: true,
				icon: path.join(root, uuid, this.iconPath + ".png"),
				width: 300,
				height: 200,
				// show: false
			});
			this.window.loadFile(path.join(root, uuid, codePath));
			this.window.once("ready-to-show", () => {
				this.window.title = this.name;
				this.window.webContents.executeJavaScript(`
				connectElgatoStreamDeckSocket(${store.get("webSocketPort")}, "${this.uuid}", "register", \`${JSON.stringify(info)}\`);
				`);
			});
		} else {
			this.process = spawn(path.join(root, uuid, codePath), [
				"-port", store.get("webSocketPort"), "-pluginUUID", this.uuid, "-registerEvent", "register", "-info", JSON.stringify(info)
			]);
		}
	}
}

class StreamDeckPluginManager {
	constructor() {
		this.pluginsDir = path.join(app.getPath("userData"), "Plugins");

		store.set("pluginsDir", this.pluginsDir);
		if (!fs.existsSync(this.pluginsDir)) fs.mkdirSync(this.pluginsDir);

		this.pluginIds = fs.readdirSync(this.pluginsDir, { withFileTypes: true })
			.filter((item) => item.isDirectory())
			.map((item) => item.name);
		this.plugins = {};
		this.server = null;
	}

	start() {
		this.pluginIds.forEach((uuid) => {
			let pl = new StreamDeckPlugin(this.pluginsDir, uuid);
			this.plugins[uuid] = pl;
		});
		
		this.server = new WebSocketServer({ port: store.get("webSocketPort") });
		this.server.on("connection", (ws) => {
			ws.on("message", (data) => {
				data = JSON.parse(data);
				if (data.event == "register") {
					this.plugins[data.uuid].socket = ws;
				}
			})
		});
	}

	async sendEvent(plugin, data) {
		this.plugins[plugin].socket.send(JSON.stringify(data));
	}

	async sendGlobalEvent(data) {
		data = JSON.stringify(data);
		Object.values(this.plugins).forEach((plugin) => {
			plugin.socket.send(data);
		});
	}
}

const pluginManager = new StreamDeckPluginManager();
pluginManager.start();
module.exports = { pluginManager };