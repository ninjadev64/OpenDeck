const { pluginManager } = require("./plugins");
const { keys } = require("./shared");

class EventHandler {
    keyDown(button) {
        pluginManager.sendEvent(keys[button].plugin, {
            event: "keyDown",
            action: keys[button].uuid,
            context: button,
            device: 0,
            payload: {
                settings: {},
                coordinates: {
                    row: Math.floor(button / 3) + 1,
                    column: button % 3
                },
                isInMultiAction: false
            }
        });
    }

    keyUp(button) {
        pluginManager.sendEvent(keys[button].plugin, {
            event: "keyUp",
            action: keys[button].uuid,
            context: button,
            device: 0,
            payload: {
                settings: {},
                coordinates: {
                    row: Math.floor(button / 3) + 1,
                    column: button % 3
                },
                isInMultiAction: false
            }
        });
    }

    willAppear(button) {
        pluginManager.sendEvent(keys[button].plugin, {
            event: "willAppear",
            action: keys[button].uuid,
            context: button,
            device: 0,
            payload: {
                controller: "Keypad",
                settings: {},
                coordinates: {
                    row: Math.floor(button / 3) + 1,
                    column: button % 3
                },
                isInMultiAction: false
            }
        });
    }

    willDisappear(button) {
        pluginManager.sendEvent(keys[button].plugin, {
            event: "willDisappear",
            action: keys[button].uuid,
            context: button,
            device: 0,
            payload: {
                controller: "Keypad",
                settings: {},
                coordinates: {
                    row: Math.floor(button / 3) + 1,
                    column: button % 3
                },
                isInMultiAction: false
            }
        });
    }
}

const eventHandler = new EventHandler();
module.exports = { eventHandler };