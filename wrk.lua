local counter = 0
local login_freq = 1000
local login_request = nil
local auth_request = nil

function init(args)
    password = args[1] or error("password required")
    login_freq = tonumber(args[2] or login_freq) or error("invalid login_freq")

    login_request = wrk.format("POST", "/auth/login", {
        ["Content-Type"] = "application/json"
    }, '{"password": "' .. password .. '"}')
end

function request()
    if counter % login_freq == 0 then
        return login_request
    else
        return auth_request
    end
end

function response(status, headers)
    if status ~= 200 then
        error("Unexpected status " .. status .. " for request " .. counter)
    end

    counter = counter + 1

    if headers["set-cookie"] then
        auth_request = wrk.format("GET", "/auth_request", {
            ["X-Original-URI"] = "/",
            ["Cookie"] = string.gsub(headers["set-cookie"], ";.*", "")
        })
    end
end
