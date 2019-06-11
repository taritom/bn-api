const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm');const debug = require("debug");var log=debug('bn-api');
const events = require('../../helpers/events');

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/events/{{start_immediately_event_id}}/ticket_types';


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

let requestBody = `{
	"name":"General Admission_{{$timestamp}}",
	"capacity": 1000,
	"start_date":null,
	"end_date": "9999-01-10T02:22:00",
	"price_in_cents": 2500,
	"limit_per_person": 50,
	"visibility": "Always"
}`;


describe('OrgMember - create tickets - start immediately', function () {
    before(async function () {
        await events.create("start_immediately_event_id", "Sales start immediately");
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


        pm.environment.set("immediate_ticket_type_id", JSON.parse(responseBody).id);

        //pm.environment.set("ga_ticket_type_id", JSON.parse(responseBody).id);
        //pm.environment.set("ticket_limit_above_max", 51);
    });

    it("should be 201", function () {
        expect(response.status).to.equal(201);
    })


});

            
