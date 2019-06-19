const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm');
const debug = require("debug");
var log = debug('bn-api');
const events = require('../../helpers/events');
const cart = require('../../helpers/cart');

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/admin/orders?event_id={{order_search_event_id}}&page=0&limit=10&dir=Asc';


var response;
var responseBody;


const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{org_admin_token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = ``;
let json = {};

describe('OrgAdmin - Search Orders Page', function () {
    before(async function () {
        this.timeout(100000);

        let event = await events.create("order_search_event_id", "Order Search test");
        await cart.createPaid(event);
        await cart.createPaid(event);
        await cart.createPaid(event);
        response = await get(requestBody);
        log(response.request.header);
        log(response.request.url);
        log(response.request._data);
        log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //log(pm);
        log(response.status);
        console.log(responseBody);

        json = JSON.parse(responseBody);
    });

    after(async function () {
        // add after methods


    });

    it("should be 200", function () {
        expect(response.status).to.equal(200);
    })


    it("orders should be present", function () {
        expect(json.data.length).to.equal(3);
    });


});

            
