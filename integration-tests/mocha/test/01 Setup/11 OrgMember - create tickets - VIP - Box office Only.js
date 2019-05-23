const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm');const debug = require("debug");var log=debug('bn-api');

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/events/{{last_event_id}}/ticket_types';


var response;
var responseBody;


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{org_member_token}}'))

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{org_member_token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = `{
	"name":"VIP_{{$timestamp}}_With_Box_Office",
	"capacity": 100,
	"start_date":"1982-02-01T02:22:00",
	"end_date": "9999-01-10T02:22:00",
	"price_in_cents": 2500,
	"limit_per_person": 0,
	"visibility": "Always",
	"ticket_pricing":[{
			"name": "Test_{{$timestamp}}",
	"price_in_cents": 3000,
	"start_date":"1982-02-01T02:22:00",
	"end_date": "2022-02-01T02:22:00"

	}, {
			"name": "Test_{{$timestamp}}_late_bird",
	"price_in_cents": 4000,
	"start_date":"2022-02-01T02:22:00",
	"end_date": "9999-01-10T02:22:00"

	},
	{
			"name": "Test_{{$timestamp}}_box_office_only",
	"price_in_cents": 8000,
	"is_box_office_only": true,
	"start_date":"2022-02-01T02:22:00",
	"end_date": "9999-01-10T02:22:00"

	}
	]
}`;


describe('OrgMember - create tickets - VIP - Box office Only', function () {
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

        pm.environment.set("last_ticket_type_id", JSON.parse(responseBody).id);


    });

    it("should be 201", function () {
        expect(response.status).to.equal(201);
    })


});

            
