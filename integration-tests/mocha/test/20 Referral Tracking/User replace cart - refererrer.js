const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm');const debug = require("debug");var log=debug('bn-api');;
const user  = require("../../helpers/user");
const events = require("../../helpers/events");
const stripe = require("../../helpers/stripe");
const cart = require("../../helpers/cart");

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/cart/checkout';


var response;
var responseBody;


const put = async function (request_body) {
    return baseUrl
        .put(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{user_token}}'))

        .send(pm.substitute(request_body));
};


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{user_token}}'))

        .send(pm.substitute(request_body));
};


let json = {};

describe('User - replace cart - referrer', function () {
    before(async function () {
        this.timeout(10000);
        await user.registerAndLogin();
        let event = await events.create();
        let ticket_type = await events.createTickets(event, "referral_ticket_type_id");
        await cart.replace(ticket_type.id);

        await stripe.getToken();


        let requestBody = `{
	"method": {
		"type" : "Card",
		"provider": "Stripe",
		"token" : "{{last_credit_card_token}}",
		"save_payment_method": false,
		"set_default": true
	},
	"tracking_data": {
	   "referrer": "http://nowhere.com" 
	}
}`;
        response = await post(requestBody);

        log(response.request.header);
        log(response.request.url);
        log(response.request._data);
        log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //log(pm);
        log(response.status);
        log(responseBody);

        json = JSON.parse(responseBody);

        pm.environment.set("last_cart_id", json.cart_id);
    });

    after(async function () {
        // add after methods


    });

    it("should be 200", function () {
        expect(response.status).to.equal(200);
    })


    it("tickets should be present", function () {
        expect(json.items[0].item_type).to.equal("Tickets");
        expect(json.items[0].quantity).to.equal(20);

        expect(json.items[0].ticket_type_id).to.equal(pm.environment.get("referral_ticket_type_id"));
    });




});

            
