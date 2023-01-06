const path = require("path");
const { app, BrowserWindow } = require("electron");
const { readdirSync, readFileSync } = require("fs");
const WebSocketServer = require("ws").Server;

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
            icon: path.join(root, uuid, this.iconPath + ".png")
            // show: false,
        });
        this.window.loadFile(path.join(root, uuid, this.htmlPath));
        this.window.webContents.executeJavaScript(`
        connectElgatoStreamDeckSocket(57116, "${this.uuid}", "register", "{}");
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

    start() {
        this.pluginIds.forEach((uuid) => {
            let pl = new StreamDeckPlugin(this.pluginsDir, uuid);
            this.plugins[uuid] = pl;
        });
    }
}

module.exports = { StreamDeckPluginManager };