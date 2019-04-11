const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm')

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
		"email" : "email{{$timestamp}}@test.com",
		"note" : "Tada"
	}
}`;


describe('Box Office - Do External Payment', function () {
    before(async function () {
        response = await post(requestBody);
        console.log(response.request.header);
        console.log(response.request.url);
        console.log(response.request._data);
        console.log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //console.log(pm);
        console.log(response.status);
        console.log(responseBody);
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

            
