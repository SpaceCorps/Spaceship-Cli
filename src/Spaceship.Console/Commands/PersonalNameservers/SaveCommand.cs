using System.ComponentModel;
using Spaceship.Console.Infrastructure;
using Spectre.Console.Cli;

namespace Spaceship.Console.Commands.PersonalNameservers;

public sealed class SaveSettings : GlobalSettings
{
    [CommandArgument(0, "<domain>")]
    [Description("Domain name")]
    public required string Domain { get; set; }

    [CommandArgument(1, "<host>")]
    [Description("Nameserver host, e.g. ns1 or ns1.example.com")]
    public required string Host { get; set; }

    [CommandOption("--ips <IPS>")]
    [Description("Comma-separated IPv4/IPv6 addresses (1-16)")]
    public required string Ips { get; set; }

    [CommandOption("--rename <HOST>")]
    [Description("New host name; the old name stops resolving")]
    public string? Rename { get; set; }
}

[Description("Create or update a personal nameserver")]
public sealed class SaveCommand : SpaceshipCommand<SaveSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, SaveSettings settings)
    {
        var ips = (settings.Ips ?? "").Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
        if (ips.Length is < 1 or > 16)
            throw new SpaceshipException("--ips needs 1-16 comma-separated addresses.");

        var current = HostLabel.Normalize(settings.Host, settings.Domain);
        var host = string.IsNullOrWhiteSpace(settings.Rename) ? current : HostLabel.Normalize(settings.Rename, settings.Domain);

        var result = await client.PutAsync($"/domains/{settings.Domain}/personal-nameservers/{current}", new { host, ips });
        return ToObject(result);
    }
}
