const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm');
const user =require('../../helpers/user');

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/cart';


var response;
var responseBody;


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{user_token}}'))

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{user_token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = `{
"items": [{
"ticket_type_id": "{{last_ticket_type_id}}",
"redemption_code": "{{last_redemption_code}}",
"quantity":30
}]}`;

let json = {};

describe('User - add to cart - redemption code Copy', function () {
    before(async function () {
        await user.registerAndLogin();
        response = await post(requestBody);
        console.log(response.request.header);
        console.log(response.request.url);
        console.log(response.request._data);
        console.log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //console.log(pm);
        console.log(response.status);
        console.log(responseBody);

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
        expect(json.items[0].quantity).to.equal(30);
        expect(json.items[0].unit_price_in_cents).to.equal(0);

        expect(json.items[0].redemption_code).to.equal(pm.environment.get("last_redemption_code"));

    });

    it("should have no per item or event fees", function () {
        expect(json.items.length).to.equal(1);

    });

    it("total should be correct", function () {
        expect(json.total_in_cents).to.equal(0);
    })


});

            
