const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm');const debug = require("debug");var log=debug('bn-api');var log=debug('bn-api');;
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


const post = async function (request_body, token) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer ' + token))

        .send(pm.substitute(request_body));
};


let json = {};

describe('Bulk create orders', function () {
    before(async function () {
        this.timeout(100000);
        let quantity = 30;
        let event = await events.create();
        let ticket_type = await events.createTickets(event, "ticket_type_id", quantity);

        let user_tokens = [];
        let promises = [];
        for(let i =0; i< quantity; i++) {
            promises.push(user.registerAndLogin().then((user_token) => {
                log(user_token);
                user_tokens[i] = user_token;
                return cart.replace(ticket_type.id, null, 1, user_token);
            }).then(()=>  {return stripe.getToken();}).then((stripe_token)=>{

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
               return  post(requestBody, user_tokens[i]);}).then((response) => {
                expect(response.status).to.equal(200);
                log(response.request.header);
                log(response.request.url);
                log(response.request._data);
                log(response.request.method);
                responseBody = JSON.stringify(response.body);
                //log(pm);
                log(response.status);
                log(responseBody);
            } ));







        }
        await Promise.all(promises);


        //json = JSON.parse(responseBody);

        //pm.environment.set("last_cart_id", json.cart_id);
    });

    after(async function () {
        // add after methods


    });

    it("should be 200", function () {
       // expect(response.status).to.equal(200);
    })







});

            
