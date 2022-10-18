const spawn = require("child_process").spawn;
const KS = require("node-key-sender");

class BasicActions {
    text(dat) {
        console.log(`Button ${dat.button} was pressed!`);
    }

    application(dat, option) {
        let child = spawn(option, [], {
            detached: true,
            stdio: [ 'ignore', 'ignore', 'ignore' ]
        });
        
        child.unref();
    }

    keyCombo(dat, option) {
        KS.sendCombination(option.split("+"));
    }
}