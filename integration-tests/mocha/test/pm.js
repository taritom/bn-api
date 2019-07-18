const fs = require('fs');
const DEFAULTS = {
    bulkOrderPaidQuantity: 10,
    bulkOrderFreeQuantity: 100,
    server: "localhost:8088",
    stripePk: "pk_test_nJGSQo5LQ7i8h8OkEjYeCqVc"
};

const pm = {};
const debug=require('debug');var log = debug('bn-api');
try {
    pm.environment = JSON.parse(fs.readFileSync("env.json"));
    // log(pm);
} catch (err) {
    log(err);
    pm.environment = {};
}

pm.environment.set = function (key, value) {
    pm.environment[key] = value;
    fs.writeFileSync("env.json", JSON.stringify(pm.environment));
};

pm.environment.get = function (key) {
    return pm.environment[key];
};

for (let key in DEFAULTS) {
    if (typeof pm.environment.get(key) === "undefined") {
        pm.environment.set(key, DEFAULTS[key]);
    }
}


pm.environment.set("$timestamp", Math.floor(Date.now() / 1000));

pm.variables = pm.environment;

pm.substitute = function (str) {
    let matches = /\{\{([^\{\}]+)\}\}/gm.exec(str);
    while (matches) {
        matches.forEach((match, groupIndex) => {
            str = str.replace("{{" + match + "}}", pm.environment.get(match));
        });
        matches = /\{\{([^\{\}]+)\}\}/gm.exec(str);
    }

    return str;
};

pm.test = it;
module.exports = pm;
