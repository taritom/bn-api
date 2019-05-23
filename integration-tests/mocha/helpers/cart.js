const {put} = require("./requests");
const pm = require('../test/pm');
const expect = require('chai').expect;
const debug = require("debug");var log=debug('bn-api');

const replace = async function (ticket_type_id, saveVarName, quantity, token) {
    let requestBody = `{
"items": [{
"ticket_type_id": "` + (ticket_type_id || "{{last_ticket_type_id}}") + `",
"quantity":${quantity || 20}
}]}`;
    log(requestBody);
    let response = await put('/cart', requestBody, token || "{{user_token}}");
    const responseBody = JSON.stringify(response.body);
    const json = JSON.parse(responseBody);
    log(responseBody);
    expect(response.status).to.equal(200);
    pm.environment.set(saveVarName || "last_cart_id", json.id);

    return json;
}

module.exports = {
    replace
}
