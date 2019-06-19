const events = require("./events");
const user = require("./user");
const stripe = require("./stripe");

const {put, post} = require("./requests");
const pm = require('../test/pm');
const expect = require('chai').expect;
const debug = require("debug");
var log = debug('bn-api');

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

const createPaid = async function (event, quantity) {
    let ticket_type = await events.createTickets(event, "___ticket_type_id", quantity || 1);

    let user_token = '';
    return await user.registerAndLogin().then((token) => {
        user_token = token;

        return replace(ticket_type.id, "__cart_id", 1, user_token);
    }).then(() => {
        return stripe.getToken();
    }).then((stripe_token) => {

        let requestBody = `{
	"method": {
		"type" : "Card",
		"provider": "Stripe",
		"token" : "${stripe_token}",
		"save_payment_method": false,
		"set_default": true
	},
	"tracking_data": {
	   "fbclid": "12123123" 
	}
}`;
        return post('/cart/checkout', requestBody, user_token);
    });
}


module.exports = {
    replace, createPaid
}
