const path = require("path");
const { app, BrowserWindow } = require("electron");
const { readdirSync, readFileSync } = require("fs");

class StreamDeckPlugin {
    constructor(dir) {
        let manifest = JSON.parse(readFileSync(path.join(dir, "manifest.json"), "utf8"));

        this.name = manifest.Name;
        this.description = manifest.Description;
        this.author = manifest.Author;
        this.version = manifest.Version;
        this.website = manifest.URL;
        this.htmlPath = manifest.CodePath;
        this.iconPath = manifest.Icon;
        this.actions = [];
        
        manifest.Actions.forEach((action) => {
            this.actions.push(new StreamDeckPluginAction(action.Name, action.UUID, action.Tooltip));
        });

        this.window = new BrowserWindow({
            autoHideMenuBar: true,
            icon: path.join(dir, this.iconPath + ".png"),
            show: false
        });
        this.window.loadFile(path.join(dir, this.htmlPath));
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
        this.pluginDirs = readdirSync(this.pluginsDir, { withFileTypes: true })
            .filter((item) => item.isDirectory())
            .map((item) => item.name);
        this.plugins = [];
    }

    start() {
        this.pluginDirs.forEach((dir) => {
            let pl = new StreamDeckPlugin(path.join(this.pluginsDir, dir));
            this.plugins.push(pl);
        });
    }
}

module.exports = { StreamDeckPluginManager };