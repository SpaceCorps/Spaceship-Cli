using System.ComponentModel;
using Spaceship.Console.Infrastructure;
using Spectre.Console.Cli;

namespace Spaceship.Console.Commands.Domains;

public sealed class EmailProtectionSettings : GlobalSettings
{
    [CommandArgument(0, "<domain>")]
    [Description("Domain name")]
    public required string Domain { get; set; }

    [CommandOption("--enable")]
    [Description("Enable the contact form")]
    public bool Enable { get; set; }

    [CommandOption("--disable")]
    [Description("Disable the contact form")]
    public bool Disable { get; set; }
}

[Description("Update domain email protection (contact form) preference")]
public sealed class EmailProtectionCommand : SpaceshipCommand<EmailProtectionSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, EmailProtectionSettings settings)
    {
        if (!settings.Enable && !settings.Disable)
            throw new SpaceshipException("Specify --enable or --disable.");
        if (settings.Enable && settings.Disable)
            throw new SpaceshipException("Cannot specify both --enable and --disable.");

        var result = await client.PutAsync($"/domains/{settings.Domain}/privacy/email-protection-preference", new
        {
            contactForm = settings.Enable
        });
        return ToObject(result);
    }
}
