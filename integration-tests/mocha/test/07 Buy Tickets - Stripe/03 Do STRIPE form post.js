const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm');const debug = require("debug");var log=debug('bn-api');

const baseUrl = supertest('https://api.stripe.com');

const apiEndPoint = '/v1/tokens';


var response;
var responseBody;


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/x-www-form-urlencoded')

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))


        .set('Accept', 'application/json')
        .send();
};

const stripePk = pm.environment.get('stripePk');
let requestBody = `email=test%40test.com&validation_type=card&referrer=https%3A%2F%2Fstripe.com%2Fdocs%2Fquickstart&card[number]=4242424242424242&card[exp_month]=12&card[exp_year]=30&card[cvc]=001&card[name]=test%40test.com&key=${stripePk}`;


describe('Do STRIPE form post', function () {
    before(async function () {
        response = await post(requestBody);
        log(response.request.header);
        log(response.request.url);
        log(response.request._data);
        log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //log(pm);
        log(response.status);
        log(responseBody);
    });

    after(async function () {
        // add after methods

        let json = JSON.parse(responseBody);
        pm.environment.set("last_credit_card_token", json.id);


    });

    it("should be 200", function () {
        expect(response.status).to.equal(200);
    })


});
