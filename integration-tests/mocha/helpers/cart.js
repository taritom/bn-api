const {put} = require("./requests");
const pm = require('../test/pm');
const expect = require('chai').expect;


const replace = async function (ticket_type_id, saveVarName) {
    let requestBody = `{
"items": [{
"ticket_type_id": "` + (ticket_type_id || "{{last_ticket_type_id}}") + `",
"quantity":20
}]}`;
    let response = await put('/cart', requestBody, '{{user_token}}');
    const responseBody = JSON.stringify(response.body);
    const json = JSON.parse(responseBody);
    expect(response.status).to.equal(200);
    pm.environment.set(saveVarName || "last_cart_id", json.id);

    return json;
}

module.exports = {
    replace
}