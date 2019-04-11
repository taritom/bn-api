const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../../pm');
const moment = require('moment');

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/reports/{{last_org_id}}?report=transaction_details&start_utc={{start_utc}}&end_utc={{end_utc}}';


var response;
var responseBody;


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{org_admin_token}}'))

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{org_admin_token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = ``;


describe('OrgAdmin - Organization', function () {
    before(async function () {

        var start_utc = moment().subtract(15, "minutes").utc().format("YYYY-MM-DDTHH:mm:ss");
        var end_utc = moment().add(15, "minutes").utc().format("YYYY-MM-DDTHH:mm:ss");
        pm.environment.set("start_utc", start_utc);
        pm.environment.set("end_utc", end_utc);
        response = await get(requestBody);
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
    it("should not be empty", function () {
        let json = JSON.parse(responseBody);

        expect(json).to.not.be.empty;
    });


});

            
