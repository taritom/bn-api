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

let requestBody = ``;

let json = {};
describe('User - view cart new order', function () {
    before(async function () {
        response = await get(requestBody);
        console.log(response.request.header);
        console.log(response.request.url);
        console.log(response.request._data);
        console.log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //console.log(pm);
        console.log(response.status);
        console.log(responseBody);

        json = JSON.parse(responseBody);
    });

    after(async function () {
        // add after methods


    });

    it("should be 200", function () {
        expect(response.status).to.equal(200);
    })


    it("tickets should be present", function () {
        expect(json.items[0].item_type).to.equal("Tickets");
        expect(json.items[0].unit_price_in_cents).to.equal(3000);
    });

    it("fees should be present", function () {
        expect(json.items[1].item_type).to.equal("PerUnitFees");
        expect(json.items[0].quantity).to.equal(2);
        expect(json.items[1].unit_price_in_cents).to.equal(10);
    });

    it("fees should be present", function () {
        expect(json.items[2].item_type).to.equal("EventFees");

        expect(json.items[2].unit_price_in_cents).to.equal(100);
    });

    it("total should be correct", function () {
        expect(json.total_in_cents).to.equal(6120);
    })


});

            
