using System.ComponentModel;
using Spaceship.Console.Infrastructure;
using Spectre.Console.Cli;

namespace Spaceship.Console.Commands.PersonalNameservers;

public sealed class DeleteSettings : GlobalSettings
{
    [CommandArgument(0, "<domain>")]
    [Description("Domain name")]
    public required string Domain { get; set; }

    [CommandArgument(1, "<host>")]
    [Description("Nameserver host, e.g. ns1 or ns1.example.com")]
    public required string Host { get; set; }
}

[Description("Delete a personal nameserver")]
public sealed class DeleteCommand : SpaceshipCommand<DeleteSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, DeleteSettings settings)
    {
        var host = HostLabel.Normalize(settings.Host, settings.Domain);
        var result = await client.DeleteAsync($"/domains/{settings.Domain}/personal-nameservers/{host}");
        return ToObject(result);
    }
}
