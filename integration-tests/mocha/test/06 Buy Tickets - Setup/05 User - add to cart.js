const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm');const debug = require("debug");var log=debug('bn-api');;
const user = require('../../helpers/user');

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
"quantity":2
}, {
"ticket_type_id": "{{vip_ticket_type_id}}",
"quantity":7
}]}`;
let json = {};

describe('User - add to cart', function () {
    before(async function () {
        await user.registerAndLogin();
        response = await put(requestBody);
        log(response.request.header);
        log(response.request.url);
        log(response.request._data);
        log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //log(pm);
        log(response.status);

        json = JSON.parse(responseBody);

        pm.environment.set("last_cart_id", json.cart_id);
        log(responseBody);
    });

    after(async function () {
        // add after methods


    });

    it("should be 200", function () {
        expect(response.status).to.equal(200);
    })


    it("tickets should be present", function () {
        expect(json.items.length).to.equal(5);
    });

    it("total should be correct", function () {
        expect(json.total_in_cents).to.equal(27190);
    });

    it("should be only 1 ticket_type with a limit", function () {
        expect(json.limited_tickets_remaining.length).to.equal(1);
    });

    it("should report 48 remaining tickets available", function () {
        expect(json.limited_tickets_remaining[0].tickets_remaining).to.equal(48);
    });


});

            
