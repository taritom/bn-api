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
	"name":"General Admission_{{$timestamp}}",
	"capacity": 1000,
	"start_date":"1982-02-01T02:22:00",
	"end_date": "2021-01-10T02:22:00",
	"price_in_cents": 3000,
	"limit_per_person": 50,
	"visibility": "Always",
	"ticket_pricing":[
		{
			"name": "Standard",
			"price_in_cents": 3000,
			"start_date":"1981-02-01T02:22:00",
			"end_date": "2022-02-01T02:22:00"

		}
	]
}`;


describe('OrgMember - create tickets - fails due to validation errors', function () {
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

    it("should be 422", function () {
        expect(response.status).to.equal(422);
    })


    it("should have validation error", function () {

        let r = JSON.parse(responseBody);
        expect(r.error).to.equal("Validation error");

        expect(r.fields["ticket_pricing.end_date"].length).to.equal(1);
        expect(r.fields["ticket_pricing.end_date"][0].code).to.equal("ticket_pricing_overlapping_ticket_type_end_date");
        expect(r.fields["ticket_pricing.end_date"][0].message).to.equal("Ticket pricing dates overlap ticket type end date");
        expect(r.fields["ticket_pricing.end_date"][0].params.end_date).to.equal("2022-02-01T02:22:00");

        expect(r.fields["ticket_pricing.start_date"].length).to.equal(1);
        expect(r.fields["ticket_pricing.start_date"][0].code).to.equal("ticket_pricing_overlapping_ticket_type_start_date");
        expect(r.fields["ticket_pricing.start_date"][0].message).to.equal("Ticket pricing dates overlap ticket type start date");
        expect(r.fields["ticket_pricing.start_date"][0].params.start_date).to.equal("1981-02-01T02:22:00");
    })


});

            
