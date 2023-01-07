const { pluginManager } = require("./plugins");
const { SerialPort } = require("serialport");
const { ReadlineParser } = require("@serialport/parser-readline");

const Store = require("electron-store");
const store = new Store();
SerialPort.list().then((ports) => { store.set("allPorts", ports); });

class SerialInterface {
    constructor() {
        this.port = new SerialPort({ path: store.get("serialPort"), baudRate: 57600 });
        this.parser = this.port.pipe(new ReadlineParser({ delimiter: "\r\n" }));

        this.buttons = {
            0: "com.elgato.template.action",
            1: "com.elgato.template.action",
            2: "com.elgato.template.action",
            3: "com.elgato.template.action",
            4: "com.elgato.template.action",
            5: "com.elgato.template.action",
            6: "com.elgato.template.action",
            7: "com.elgato.template.action",
            8: "com.elgato.template.action",
            9: "com.elgato.template.action"
        }

        this.parser.on("data", (data) => {
            data = JSON.parse(data);
            if (data.button === 0) return;

            let action = this.buttons[data.button];
            let plugin = action.substring(0, action.lastIndexOf("."));

            pluginManager.sendEvent(plugin, 
                {
                    event: "keyDown",
                    action: action,
                    context: data.button,
                    device: 0,
                    payload: {
                        settings: {},
                        coordinates: {
                            row: Math.floor(data.button / 3) + 1,
                            column: data.button % 3
                        },
                        isInMultiAction: false
                    }
                }
            );
        });
    }
}

const serialInterface = new SerialInterface();
module.exports = { serialInterface };