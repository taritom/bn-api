const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/cart';


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

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{user_token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = `{
"items": [{
"ticket_type_id": "{{ga_ticket_type_id}}",
"quantity":20
}]}`;
let json = {};

describe('User - replace cart', function () {
    before(async function () {
        response = await put(requestBody);
        console.log(response.request.header);
        console.log(response.request.url);
        console.log(response.request._data);
        console.log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //console.log(pm);
        console.log(response.status);
        console.log(responseBody);

        json = JSON.parse(responseBody);

        pm.environment.set("last_cart_id", json.id);
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
        expect(json.items[0].unit_price_in_cents).to.equal(3000);

        expect(json.items[0].ticket_type_id).to.equal(pm.environment.get("ga_ticket_type_id"));
    });

    it("fees should be present", function () {
        expect(json.items[1].item_type).to.equal("PerUnitFees");
        expect(json.items[1].quantity).to.equal(20);
        expect(json.items[1].unit_price_in_cents).to.equal(10);
    });

    it("fees should be present", function () {
        expect(json.items[2].item_type).to.equal("EventFees");

        expect(json.items[2].unit_price_in_cents).to.equal(100);
    });

    it("total should be correct", function () {
        expect(json.total_in_cents).to.equal(60300);
    })


});

            
