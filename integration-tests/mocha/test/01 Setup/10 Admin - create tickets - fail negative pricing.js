const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/events/{{last_event_id}}/ticket_types';


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

let requestBody = `{
	"name":"VIP_{{$timestamp}}_{{$randomInt}}",
	"capacity": 100,
	"start_date":"1982-02-01T02:22:00",
	"end_date": "9999-01-10T02:22:00",
	"price_in_cents": -3000,
	"limit_per_person": 0,
	"sold_out_behavior": "ShowSoldOut",
	"is_private": false,
	"ticket_pricing":[{
			"name": "Test_{{$timestamp}}",
	"price_in_cents": -3000,
	"start_date":"1982-02-01T02:22:00",
	"end_date": "2022-02-01T02:22:00"

	}, {
			"name": "Test_{{$timestamp}}_late_bird",
	"price_in_cents": -4000,
	"start_date":"2022-02-01T02:22:00",
	"end_date": "9999-01-10T02:22:00"

	}
	]
}`;


describe('Admin - create tickets - fail negative pricing', function () {
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

    it("should be 422", function () {
        expect(response.status).to.equal(422); // Unprocessable Entity
    });


    it("should have correct fields errors", function () {

        let r = JSON.parse(responseBody);
        expect(r.fields["ticket_pricing.price_in_cents"].length).to.equal(1);
        expect(r.fields["ticket_pricing.price_in_cents"][0].code).to.equal("number_must_be_positive");
        expect(r.fields["ticket_pricing.price_in_cents"][0].message.length).to.be.greaterThan(0);

    })


});

            
