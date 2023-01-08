const path = require("path");
const { app, BrowserWindow } = require("electron");
const { readdirSync, readFileSync } = require("fs");
const WebSocketServer = require("ws").Server;

const os = require("os");
const { version } = require("./package.json");

class StreamDeckPlugin {
    constructor(root, uuid) {
        let manifest = JSON.parse(readFileSync(path.join(root, uuid, "manifest.json"), "utf8"));

        this.uuid = uuid;
        this.name = manifest.Name;
        this.description = manifest.Description;
        this.author = manifest.Author;
        this.version = manifest.Version;
        this.website = manifest.URL;
        this.htmlPath = manifest.CodePath;
        this.iconPath = manifest.Icon;
        this.actions = [];
        this.socket = null;
        
        manifest.Actions.forEach((action) => {
            this.actions.push(new StreamDeckPluginAction(action.Name, action.UUID, action.Tooltip));
        });

        this.window = new BrowserWindow({
            autoHideMenuBar: true,
            icon: path.join(root, uuid, this.iconPath + ".png"),
            width: 300,
            height: 200,
            // show: false
        });
        this.window.loadFile(path.join(root, uuid, this.htmlPath));
        this.window.once("ready-to-show", () => {
            this.window.title = this.name;
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
        this.window.webContents.executeJavaScript(`
        connectElgatoStreamDeckSocket(57116, "${this.uuid}", "register", \`${JSON.stringify(info)}\`);
        `);
    }
}

class StreamDeckPluginAction {
    constructor(name, uuid, tooltip) {
        this.name = name;
        this.uuid = uuid;
        this.tooltip = tooltip;
    }
}

class StreamDeckPluginManager {
    constructor() {
        this.pluginsDir = path.join(app.getPath("userData"), "Plugins");
        this.pluginIds = readdirSync(this.pluginsDir, { withFileTypes: true })
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
        
        this.server = new WebSocketServer({ port: 57116 });
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
        this.plugins[plugin].socket.send(data);
    }
}

const pluginManager = new StreamDeckPluginManager();
pluginManager.start();
module.exports = { pluginManager };