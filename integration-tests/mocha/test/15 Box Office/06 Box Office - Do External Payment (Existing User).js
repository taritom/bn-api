const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm');const debug = require("debug");var log=debug('bn-api');

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/cart/checkout';


var response;
var responseBody;


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{org_boxoffice_token}}'))

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{org_boxoffice_token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = `{
	"amount": 8000,
	"method": {
		"type" : "External",
		"reference": "INV{{$timestamp}}",
		"external_payment_type": "Cash",
		"first_name" : "Name{{$timestamp}}",
		"last_name" : "LastName{{$timestamp}}",
		"email" : "{{last_email}}",
		"note" : "Tada"
	}
}`;


describe('Box Office - Do External Payment (Existing User)', function () {
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


    });

    it("should be 200", function () {
        expect(response.status).to.equal(200);
    })


    it("should be paid", function () {

        let json = JSON.parse(responseBody);
        expect(json.status).to.equal("Paid");
        pm.environment.set("last_box_office_user_id", json.user_id);
    })


});

            
