const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/events';


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
    "name": "It's my party",
    "organization_id": "{{last_org_id}}",
    "venue_id": "{{last_venue_id}}",
    "event_start": "2059-11-13T12:00:00",
    "is_external": true,
    "external_url": "https://www.eventbrite.com/e/why-cloud-why-xero-why-now-johannesburg-limited-seats-tickets-52952818305?aff=ebdshpmoodssection",
    "event_type": "Music"
}`;


describe('BoxOffice - Create Event That is External - 401', function () {
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

    it("should be 401", function () {
        expect(response.status).to.equal(401);
    })


});

            
