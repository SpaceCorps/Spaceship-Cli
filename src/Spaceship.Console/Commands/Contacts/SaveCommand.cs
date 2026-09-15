using System.ComponentModel;
using Spaceship.Console.Infrastructure;
using Spectre.Console.Cli;

namespace Spaceship.Console.Commands.Contacts;

public sealed class SaveSettings : GlobalSettings
{
    [CommandOption("--first-name <NAME>")]
    [Description("First name")]
    public required string FirstName { get; set; }

    [CommandOption("--last-name <NAME>")]
    [Description("Last name")]
    public required string LastName { get; set; }

    [CommandOption("--email <EMAIL>")]
    [Description("Email address")]
    public required string Email { get; set; }

    [CommandOption("--address <ADDRESS>")]
    [Description("Primary address")]
    public required string Address { get; set; }

    [CommandOption("--city <CITY>")]
    [Description("City")]
    public required string City { get; set; }

    [CommandOption("--country <CODE>")]
    [Description("ISO 3166-1 alpha-2 country code")]
    public required string Country { get; set; }

    [CommandOption("--phone <PHONE>")]
    [Description("Phone number as +<country code>.<number>, e.g. +46.701234567")]
    public required string Phone { get; set; }

    [CommandOption("--phone-ext <EXT>")]
    [Description("Phone extension")]
    public string? PhoneExt { get; set; }

    [CommandOption("--fax <FAX>")]
    [Description("Fax number, same format as --phone")]
    public string? Fax { get; set; }

    [CommandOption("--fax-ext <EXT>")]
    [Description("Fax extension")]
    public string? FaxExt { get; set; }

    [CommandOption("--tax-number <NUMBER>")]
    [Description("Tax number")]
    public string? TaxNumber { get; set; }

    [CommandOption("--organization <ORG>")]
    [Description("Organization name")]
    public string? Organization { get; set; }

    [CommandOption("--address2 <ADDRESS>")]
    [Description("Secondary address")]
    public string? Address2 { get; set; }

    [CommandOption("--state <STATE>")]
    [Description("State or province")]
    public string? State { get; set; }

    [CommandOption("--postal-code <CODE>")]
    [Description("Postal code")]
    public string? PostalCode { get; set; }
}

[Description("Save contact details")]
public sealed class SaveCommand : SpaceshipCommand<SaveSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, SaveSettings settings)
    {
        if (string.IsNullOrWhiteSpace(settings.Phone))
            throw new SpaceshipException("--phone is required.");

        var body = new Dictionary<string, object?>
        {
            ["firstName"] = settings.FirstName,
            ["lastName"] = settings.LastName,
            ["email"] = settings.Email,
            ["address1"] = settings.Address,
            ["city"] = settings.City,
            ["country"] = settings.Country,
            ["phone"] = settings.Phone,
            ["phoneExt"] = settings.PhoneExt,
            ["fax"] = settings.Fax,
            ["faxExt"] = settings.FaxExt,
            ["taxNumber"] = settings.TaxNumber,
            ["organization"] = settings.Organization,
            ["address2"] = settings.Address2,
            ["stateProvince"] = settings.State,
            ["postalCode"] = settings.PostalCode
        };

        var result = await client.PutAsync("/contacts", body);
        return ToObject(result);
    }
}
