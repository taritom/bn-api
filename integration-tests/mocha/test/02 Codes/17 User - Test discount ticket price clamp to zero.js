const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm');const debug = require("debug");var log=debug('bn-api');

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/cart';


var response;
var responseBody;


const put = async function (request_body) {
    return baseUrl
        .put(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{discount_user_token}}'))

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{discount_user_token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = `{
"items": [{
"ticket_type_id": "{{ga_ticket_type_id}}",
"redemption_code": "{{discount_redemption_code_clamped}}",
"quantity":1
}, {
"ticket_type_id": "{{vip_ticket_type_id}}",
"quantity":1,
"redemption_code": "{{discount_percentage_redemption_code_clamped}}"
}
]}`;

let json = {};
describe('User - Test discount ticket price clamp to zero', function () {
    before(async function () {
        response = await put(requestBody);
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
        expect(json.items.length).to.equal(4);

        // these are tested properly in later calls

    });

    it("Discount prices should be clamped at -3000", function () {
        expect(json.items[2].unit_price_in_cents).to.equal(-3000);
        expect(json.items[3].unit_price_in_cents).to.equal(-3000);
    })


});

            
