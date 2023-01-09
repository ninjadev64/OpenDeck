const { pluginManager } = require("./plugins");
const { keys } = require("./shared");

class EventHandler {
    keyDown(button) {
        pluginManager.sendEvent(keys[button].plugin, 
            {
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
            }
        );
    }

    keyUp(button) {
        pluginManager.sendEvent(keys[button].plugin, 
            {
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
            }
        );
    }
}

const eventHandler = new EventHandler();
module.exports = { eventHandler };