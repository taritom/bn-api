const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm');const debug = require("debug");var log=debug('bn-api');

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/orders/{{last_cart_id}}/details';


var response;
var responseBody;


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{token}}'))

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = ``;
let json = {};


describe('Admin - view order details all refunded', function () {
    before(async function () {
        response = await get(requestBody);
        log(response.request.header);
        log(response.request.url);
        log(response.request._data);
        log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //log(pm);
        log(response.status);
        log(responseBody);

        json = JSON.parse(responseBody);
    });

    after(async function () {
        // add after methods


    });

    it("should be 200", function () {
        expect(response.status).to.equal(200);
    })


    it("no tickets from other organizations", function () {
        expect(json.order_contains_other_tickets).to.equal(false);
    });

    it("correct number of order items", function () {
        expect(json.items.length).to.equal(3);
    });

    it("tickets should be present", function () {
        expect(json.items[0].ticket_price_in_cents).to.equal(0);
        expect(json.items[0].fees_price_in_cents).to.equal(0);
        expect(json.items[0].total_price_in_cents).to.equal(0);
        expect(json.items[0].status).to.equal('Refunded');
        expect(json.items[0].refundable).to.equal(false);

        expect(json.items[1].ticket_price_in_cents).to.equal(0);
        expect(json.items[1].fees_price_in_cents).to.equal(0);
        expect(json.items[1].total_price_in_cents).to.equal(0);
        expect(json.items[1].status).to.equal('Refunded');
        expect(json.items[1].refundable).to.equal(false);
    });

    it("event fees should be present", function () {
        expect(json.items[2].ticket_price_in_cents).to.equal(0);
        expect(json.items[2].fees_price_in_cents).to.equal(0);
        expect(json.items[2].total_price_in_cents).to.equal(0);
        expect(json.items[2].status).to.equal('Refunded');
        expect(json.items[2].refundable).to.equal(false);
    });


});

            
