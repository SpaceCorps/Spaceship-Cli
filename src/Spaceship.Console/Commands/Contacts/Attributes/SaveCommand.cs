using System.ComponentModel;
using Spaceship.Console.Infrastructure;
using Spectre.Console.Cli;

namespace Spaceship.Console.Commands.Contacts.Attributes;

public sealed class SaveSettings : GlobalSettings
{
    [CommandOption("--file <FILE>")]
    [Description("JSON file with the attributes, including \"type\" (or pipe via stdin)")]
    public string? File { get; set; }
}

[Description("Save contact attributes; returns the attributes ID")]
public sealed class SaveCommand : SpaceshipCommand<SaveSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, SaveSettings settings)
    {
        var body = await JsonInput.ReadAsync(settings.File, "{\"type\": \"...\", ...}");
        var result = await client.PutAsync("/contacts/attributes", ToObject(body));
        return ToObject(result);
    }
}
