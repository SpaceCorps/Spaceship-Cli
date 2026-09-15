using System.Reflection;
using Spectre.Console.Cli;

var version = typeof(Program).Assembly
    .GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion.Split('+')[0] ?? "0.0.0";

var app = new CommandApp();

app.Configure(config =>
{
    config.SetApplicationName("spaceship");
    config.SetApplicationVersion(version);

    config.AddBranch("domains", domains =>
    {
        domains.SetDescription("Domain management");
        domains.AddCommand<Spaceship.Console.Commands.Domains.ListCommand>("list");
        domains.AddCommand<Spaceship.Console.Commands.Domains.GetCommand>("get");
        domains.AddCommand<Spaceship.Console.Commands.Domains.CheckCommand>("check");
        domains.AddCommand<Spaceship.Console.Commands.Domains.CheckBatchCommand>("check-batch");
        domains.AddCommand<Spaceship.Console.Commands.Domains.RegisterCommand>("register");
        domains.AddCommand<Spaceship.Console.Commands.Domains.DeleteCommand>("delete");
        domains.AddCommand<Spaceship.Console.Commands.Domains.RenewCommand>("renew");
        domains.AddCommand<Spaceship.Console.Commands.Domains.RestoreCommand>("restore");
        domains.AddCommand<Spaceship.Console.Commands.Domains.AutorenewCommand>("autorenew");
        domains.AddCommand<Spaceship.Console.Commands.Domains.NameserversCommand>("nameservers");
        domains.AddCommand<Spaceship.Console.Commands.Domains.ContactsCommand>("contacts");
        domains.AddCommand<Spaceship.Console.Commands.Domains.PrivacyCommand>("privacy");
        domains.AddCommand<Spaceship.Console.Commands.Domains.EmailProtectionCommand>("email-protection");
        domains.AddCommand<Spaceship.Console.Commands.Domains.AuthCodeCommand>("auth-code");
        domains.AddCommand<Spaceship.Console.Commands.Domains.TransferCommand>("transfer");
        domains.AddCommand<Spaceship.Console.Commands.Domains.TransferStatusCommand>("transfer-status");
        domains.AddCommand<Spaceship.Console.Commands.Domains.TransferLockCommand>("transfer-lock");

        domains.AddBranch("personal-ns", ns =>
        {
            ns.SetDescription("Personal nameservers (glue records)");
            ns.AddCommand<Spaceship.Console.Commands.PersonalNameservers.ListCommand>("list");
            ns.AddCommand<Spaceship.Console.Commands.PersonalNameservers.SaveCommand>("save");
            ns.AddCommand<Spaceship.Console.Commands.PersonalNameservers.DeleteCommand>("delete");
        });
    });

    config.AddBranch("dns", dns =>
    {
        dns.SetDescription("DNS record management");
        dns.AddCommand<Spaceship.Console.Commands.Dns.ListCommand>("list");
        dns.AddCommand<Spaceship.Console.Commands.Dns.SaveCommand>("save");
        dns.AddCommand<Spaceship.Console.Commands.Dns.DeleteCommand>("delete");
    });

    config.AddBranch("contacts", contacts =>
    {
        contacts.SetDescription("Contact management");
        contacts.AddCommand<Spaceship.Console.Commands.Contacts.SaveCommand>("save");
        contacts.AddCommand<Spaceship.Console.Commands.Contacts.GetCommand>("get");

        contacts.AddBranch("attributes", attributes =>
        {
            attributes.SetDescription("Contact attributes (registry-specific data)");
            attributes.AddCommand<Spaceship.Console.Commands.Contacts.Attributes.SaveCommand>("save");
            attributes.AddCommand<Spaceship.Console.Commands.Contacts.Attributes.GetCommand>("get");
        });
    });

    config.AddBranch("sellerhub", sellerhub =>
    {
        sellerhub.SetDescription("SellerHub management");
        sellerhub.AddCommand<Spaceship.Console.Commands.SellerHub.ListCommand>("list");
        sellerhub.AddCommand<Spaceship.Console.Commands.SellerHub.GetCommand>("get");
        sellerhub.AddCommand<Spaceship.Console.Commands.SellerHub.CreateCommand>("create");
        sellerhub.AddCommand<Spaceship.Console.Commands.SellerHub.UpdateCommand>("update");
        sellerhub.AddCommand<Spaceship.Console.Commands.SellerHub.DeleteCommand>("delete");
        sellerhub.AddCommand<Spaceship.Console.Commands.SellerHub.CheckoutCommand>("checkout");
        sellerhub.AddCommand<Spaceship.Console.Commands.SellerHub.VerificationCommand>("verification");
        sellerhub.AddCommand<Spaceship.Console.Commands.SellerHub.SoldCommand>("sold");

        sellerhub.AddBranch("safepay", safepay =>
        {
            safepay.SetDescription("SafePay (escrow) transactions");
            safepay.AddCommand<Spaceship.Console.Commands.SellerHub.Safepay.ListCommand>("list");
            safepay.AddCommand<Spaceship.Console.Commands.SellerHub.Safepay.GetCommand>("get");
            safepay.AddCommand<Spaceship.Console.Commands.SellerHub.Safepay.CreateCommand>("create");
        });
    });

    config.AddBranch("hyperlift", hyperlift =>
    {
        hyperlift.SetDescription("Hyperlift application hosting");
        hyperlift.AddCommand<Spaceship.Console.Commands.Hyperlift.ListCommand>("list");
        hyperlift.AddCommand<Spaceship.Console.Commands.Hyperlift.GetCommand>("get");
        hyperlift.AddCommand<Spaceship.Console.Commands.Hyperlift.BuildCommand>("build");
        hyperlift.AddCommand<Spaceship.Console.Commands.Hyperlift.BuildLogsCommand>("build-logs");
        hyperlift.AddCommand<Spaceship.Console.Commands.Hyperlift.LogsCommand>("logs");
        hyperlift.AddCommand<Spaceship.Console.Commands.Hyperlift.MetricsCommand>("metrics");
        hyperlift.AddCommand<Spaceship.Console.Commands.Hyperlift.RestartCommand>("restart");
        hyperlift.AddCommand<Spaceship.Console.Commands.Hyperlift.ScaleCommand>("scale");

        hyperlift.AddBranch("env", env =>
        {
            env.SetDescription("Environment variables");
            env.AddCommand<Spaceship.Console.Commands.Hyperlift.EnvGetCommand>("get");
            env.AddCommand<Spaceship.Console.Commands.Hyperlift.EnvSetCommand>("set");
        });
    });

    config.AddBranch("operations", operations =>
    {
        operations.SetDescription("Async operation tracking");
        operations.AddCommand<Spaceship.Console.Commands.Operations.GetCommand>("get");
    });
});

return app.Run(args);
