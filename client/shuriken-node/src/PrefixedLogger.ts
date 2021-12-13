import { Logger } from 'winston';

export class PrefixedLogger {
    private logger: Logger;
    private prefix: string;
    constructor(logger: Logger, prefix: string) {
        this.logger = logger;
        this.prefix = prefix;
    }
    public log(message: string): void {
        this.logger.log({
            level: 'info',
            message: this.prefix + ' ' + message,
        });
    }
}
